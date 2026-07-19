"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from netdiag import Command, UsageError, banner, parse_command, section, usage


class NetdiagTest(unittest.TestCase):
    def test_parses_scan_with_host_and_ports(self) -> None:
        self.assertEqual(
            parse_command(["scan", "example.com", "1-1024"]),
            Command("scan", host="example.com", ports="1-1024"),
        )

    def test_parses_ping_and_trace_with_defaults_and_overrides(self) -> None:
        self.assertEqual(parse_command(["ping", "h"]).count, 4)
        self.assertEqual(parse_command(["ping", "h", "7"]).count, 7)
        self.assertEqual(parse_command(["trace", "h"]).max_hops, 30)

    def test_rejects_unknown_and_missing_arguments(self) -> None:
        with self.assertRaises(UsageError):
            parse_command([])
        with self.assertRaises(UsageError) as ctx:
            parse_command(["bogus"])
        self.assertIn("unknown command 'bogus'", str(ctx.exception))
        with self.assertRaises(UsageError):
            parse_command(["scan", "h"])  # missing ports

    def test_usage_lists_all_three_subcommands(self) -> None:
        u = usage()
        for want in ("netdiag scan", "netdiag ping", "netdiag trace"):
            self.assertIn(want, u)

    def test_renders_the_banner(self) -> None:
        lines = banner("scan example.com").split("\n")
        self.assertEqual(len(lines), 3)
        self.assertEqual(lines[0], "╔" + "═" * 46 + "╗")
        self.assertEqual(lines[2], "╚" + "═" * 46 + "╝")
        self.assertTrue(lines[1].startswith("║  netdiag :: scan example.com"))
        self.assertTrue(lines[1].endswith("║"))
        self.assertEqual(len(lines[1]), 48)

    def test_renders_a_section_rule(self) -> None:
        rule = section("open ports")
        self.assertTrue(rule.startswith("── open ports "))
        self.assertEqual(len(rule), 48)
        self.assertTrue(rule.endswith("─────"))


if __name__ == "__main__":
    unittest.main()
