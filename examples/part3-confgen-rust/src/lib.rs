//! Config generator — Rust implementation of the Part 3 project. Reads a
//! `key = value` service spec and generates an nginx reverse-proxy block plus a
//! systemd unit. All three language implementations emit byte-identical output
//! for the same spec, verified by golden tests and a CLI diff.
//!
//! ```text
//! cargo test
//! cargo run -- service.conf
//! ```

/// A validated service spec.
#[derive(Debug, PartialEq, Eq)]
pub struct Spec {
    pub name: String,
    pub domain: String,
    pub port: u16,
    pub replicas: u8,
}

/// Spec-file problems, precise enough to fix from the message alone.
#[derive(Debug, PartialEq, Eq)]
pub enum SpecError {
    Missing(&'static str),
    Invalid { key: &'static str, value: String },
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecError::Missing(key) => write!(f, "missing required key {key}"),
            SpecError::Invalid { key, value } => {
                write!(f, "invalid value {value:?} for key {key}")
            }
        }
    }
}

impl std::error::Error for SpecError {}

/// Parses the `key = value` spec format (comments with #, blank lines ok).
pub fn parse_spec(text: &str) -> Result<Spec, SpecError> {
    let mut name = None;
    let mut domain = None;
    let mut port = None;
    let mut replicas = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue; // tolerated: not a key=value line
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "name" => name = Some(value.to_string()),
            "domain" => domain = Some(value.to_string()),
            "port" => {
                port = Some(value.parse().map_err(|_| SpecError::Invalid {
                    key: "port",
                    value: value.to_string(),
                })?)
            }
            "replicas" => {
                replicas = Some(value.parse().map_err(|_| SpecError::Invalid {
                    key: "replicas",
                    value: value.to_string(),
                })?)
            }
            _ => {} // unknown keys ignored (forward compatibility)
        }
    }

    let spec = Spec {
        name: name.ok_or(SpecError::Missing("name"))?,
        domain: domain.ok_or(SpecError::Missing("domain"))?,
        port: port.ok_or(SpecError::Missing("port"))?,
        replicas: replicas.unwrap_or(1),
    };
    if spec.replicas == 0 {
        return Err(SpecError::Invalid {
            key: "replicas",
            value: "0".into(),
        });
    }
    Ok(spec)
}

/// Renders the nginx upstream + server block.
pub fn render_nginx(spec: &Spec) -> String {
    let mut servers = String::new();
    for i in 0..spec.replicas {
        servers.push_str(&format!(
            "    server 127.0.0.1:{};\n",
            spec.port + u16::from(i)
        ));
    }
    format!(
        "upstream {name} {{\n{servers}}}\n\nserver {{\n    listen 80;\n    server_name {domain};\n\n    location / {{\n        proxy_pass http://{name};\n        proxy_set_header Host $host;\n        proxy_set_header X-Real-IP $remote_addr;\n    }}\n}}\n",
        name = spec.name,
        domain = spec.domain,
    )
}

/// Renders the systemd unit template (`name@.service` style, %i = instance).
pub fn render_systemd(spec: &Spec) -> String {
    format!(
        "[Unit]\nDescription={name} service (instance %i)\nAfter=network.target\n\n[Service]\nExecStart=/usr/local/bin/{name} --port %i\nRestart=on-failure\nUser={name}\nNoNewPrivileges=true\n\n[Install]\nWantedBy=multi-user.target\n",
        name = spec.name,
    )
}

/// Full CLI output: both artifacts, with headers (shared across languages).
pub fn generate(text: &str) -> Result<String, SpecError> {
    let spec = parse_spec(text)?;
    Ok(format!(
        "--- nginx: {name}.conf\n{nginx}\n--- systemd: {name}@.service\n{unit}",
        name = spec.name,
        nginx = render_nginx(&spec),
        unit = render_systemd(&spec),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str =
        "# demo service\nname = api\ndomain = api.example.com\nport = 8080\nreplicas = 2\n";

    #[test]
    fn parses_a_full_spec() {
        assert_eq!(
            parse_spec(SPEC),
            Ok(Spec {
                name: "api".into(),
                domain: "api.example.com".into(),
                port: 8080,
                replicas: 2,
            })
        );
    }

    #[test]
    fn missing_and_invalid_keys_are_precise_errors() {
        assert_eq!(
            parse_spec("domain = x\nport = 1\n"),
            Err(SpecError::Missing("name"))
        );
        assert_eq!(
            parse_spec("name = a\ndomain = x\nport = banana\n"),
            Err(SpecError::Invalid {
                key: "port",
                value: "banana".into()
            })
        );
        assert_eq!(
            parse_spec("name = a\ndomain = x\nport = 1\nreplicas = 0\n"),
            Err(SpecError::Invalid {
                key: "replicas",
                value: "0".into()
            })
        );
    }

    #[test]
    fn replicas_defaults_to_one() {
        let spec = parse_spec("name = a\ndomain = x\nport = 9000\n").unwrap();
        assert_eq!(spec.replicas, 1);
    }

    // GOLDEN TEST: the exact expected artifact, byte for byte. If a template
    // change is intentional, the golden text is updated in the same commit.
    #[test]
    fn nginx_golden() {
        let spec = parse_spec(SPEC).unwrap();
        let expected = "\
upstream api {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
}

server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
";
        assert_eq!(render_nginx(&spec), expected);
    }

    #[test]
    fn systemd_golden() {
        let spec = parse_spec(SPEC).unwrap();
        let expected = "\
[Unit]
Description=api service (instance %i)
After=network.target

[Service]
ExecStart=/usr/local/bin/api --port %i
Restart=on-failure
User=api
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
";
        assert_eq!(render_systemd(&spec), expected);
    }
}
