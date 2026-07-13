"""A dependency-injection container: register services by name with a factory,
then resolve them — the framework wires the graph, calling each factory and
feeding it whatever it asks the container to resolve.

This is the inversion of control from Part 2's Repository & DI lesson, made into
a reusable framework. Three things a real container must get right and this one
does, all tested:

* **Lifetimes.** A *transient* service is rebuilt on every resolve; a
  *singleton* is built once and cached.
* **Cycle detection.** If A needs B and B needs A, naive resolution recurses
  forever; we track the resolution stack and raise a clear error.
* **Missing dependencies fail loudly**, naming what was not found.

Factories build strings so the assembled graph is directly assertable.
"""

from __future__ import annotations

from collections.abc import Callable

# A factory builds a service, using the Container to resolve its own
# dependencies.
Factory = Callable[["Container"], str]


class ContainerError(Exception):
    """Raised for an unknown service or a dependency cycle."""


class Container:
    def __init__(self) -> None:
        self._factories: dict[str, tuple[Factory, bool]] = {}
        self._cache: dict[str, str] = {}
        self._resolving: list[str] = []

    def register(self, name: str, factory: Factory) -> None:
        """Register a transient service: its factory runs on every resolve,
        producing a fresh value each time."""
        self._factories[name] = (factory, False)

    def register_singleton(self, name: str, factory: Factory) -> None:
        """Register a singleton service: its factory runs at most once; the
        result is cached and returned on every later resolve."""
        self._factories[name] = (factory, True)

    def resolve(self, name: str) -> str:
        """Return the cached singleton if present, otherwise run the factory
        (which may resolve further dependencies), caching the result if it is a
        singleton.

        Raises :class:`ContainerError` if the name is not registered, or if
        resolving it would form a cycle (A -> B -> A).
        """
        if name in self._cache:
            return self._cache[name]

        if name in self._resolving:
            chain = " -> ".join([*self._resolving, name])
            raise ContainerError(f"dependency cycle: {chain}")

        if name not in self._factories:
            raise ContainerError(f"service not registered: {name}")

        factory, singleton = self._factories[name]
        self._resolving.append(name)
        try:
            value = factory(self)
        finally:
            self._resolving.pop()

        if singleton:
            self._cache[name] = value
        return value


if __name__ == "__main__":
    c = Container()
    c.register_singleton("config", lambda _c: "Config(env=prod)")
    c.register_singleton("db", lambda c: f"Pool(from {c.resolve('config')})")
    c.register("user_repo", lambda c: f"UserRepo(on {c.resolve('db')})")
    c.register("handler", lambda c: f"Handler(with {c.resolve('user_repo')})")

    print("resolve handler:")
    print(" ", c.resolve("handler"))
    print("resolve handler again (db/config are singletons, reused):")
    print(" ", c.resolve("handler"))

    bad = Container()
    bad.register("a", lambda c: f"A({c.resolve('b')})")
    bad.register("b", lambda c: f"B({c.resolve('a')})")
    print("\nresolving a cyclic graph:")
    try:
        print("  unexpectedly built:", bad.resolve("a"))
    except ContainerError as e:
        print("  refused:", e)
