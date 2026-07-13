"""Tests: the same eight scenarios as the Rust and Go twins."""

import unittest

from container import Container, ContainerError


class ContainerTest(unittest.TestCase):
    def test_resolves_a_leaf_service(self) -> None:
        c = Container()
        c.register("config", lambda _c: "Config(db=memory)")
        self.assertEqual(c.resolve("config"), "Config(db=memory)")

    def test_resolves_a_dependency_chain(self) -> None:
        c = Container()
        c.register("config", lambda _c: "Config")
        c.register("repo", lambda c: f"Repo(uses {c.resolve('config')})")
        c.register("service", lambda c: f"Service(uses {c.resolve('repo')})")
        self.assertEqual(c.resolve("service"), "Service(uses Repo(uses Config))")

    def test_unknown_service_errors_with_its_name(self) -> None:
        c = Container()
        with self.assertRaises(ContainerError) as ctx:
            c.resolve("nope")
        self.assertIn("nope", str(ctx.exception))

    def test_transient_rebuilds_every_resolve(self) -> None:
        count = 0

        def factory(_c: Container) -> str:
            nonlocal count
            count += 1
            return f"instance-{count}"

        c = Container()
        c.register("id", factory)
        self.assertEqual(c.resolve("id"), "instance-1")
        self.assertEqual(c.resolve("id"), "instance-2")
        self.assertEqual(c.resolve("id"), "instance-3")

    def test_singleton_builds_once_and_caches(self) -> None:
        count = 0

        def factory(_c: Container) -> str:
            nonlocal count
            count += 1
            return f"instance-{count}"

        c = Container()
        c.register_singleton("id", factory)
        self.assertEqual(c.resolve("id"), "instance-1")
        self.assertEqual(c.resolve("id"), "instance-1")  # cached
        self.assertEqual(count, 1)

    def test_direct_cycle_is_detected(self) -> None:
        c = Container()
        c.register("a", lambda c: f"A({c.resolve('b')})")
        c.register("b", lambda c: f"B({c.resolve('a')})")
        with self.assertRaises(ContainerError) as ctx:
            c.resolve("a")
        self.assertIn("cycle", str(ctx.exception))
        self.assertIn("a -> b -> a", str(ctx.exception))

    def test_self_cycle_is_detected(self) -> None:
        c = Container()
        c.register("loop", lambda c: c.resolve("loop"))
        with self.assertRaises(ContainerError) as ctx:
            c.resolve("loop")
        self.assertIn("cycle", str(ctx.exception))

    def test_singleton_dependency_is_shared_across_consumers(self) -> None:
        count = 0

        def db(_c: Container) -> str:
            nonlocal count
            count += 1
            return f"DB#{count}"

        c = Container()
        c.register_singleton("db", db)
        c.register("users", lambda c: f"Users({c.resolve('db')})")
        c.register("orders", lambda c: f"Orders({c.resolve('db')})")
        self.assertEqual(c.resolve("users"), "Users(DB#1)")
        self.assertEqual(c.resolve("orders"), "Orders(DB#1)")
        self.assertEqual(count, 1)


if __name__ == "__main__":
    unittest.main()
