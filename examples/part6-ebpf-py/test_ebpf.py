"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from ebpf import bucket, bucket_range, render_histogram, tally


class EbpfTest(unittest.TestCase):
    def test_buckets_follow_powers_of_two(self) -> None:
        cases = {0: 0, 1: 0, 2: 1, 3: 1, 4: 2, 7: 2, 8: 3, 1023: 9, 1024: 10}
        for v, want in cases.items():
            self.assertEqual(bucket(v), want, f"bucket({v})")

    def test_bucket_ranges_are_inclusive_powers_of_two(self) -> None:
        self.assertEqual(bucket_range(0), (0, 1))
        self.assertEqual(bucket_range(1), (2, 3))
        self.assertEqual(bucket_range(2), (4, 7))
        self.assertEqual(bucket_range(4), (16, 31))

    def test_tally_bins_samples_by_bucket(self) -> None:
        self.assertEqual(tally([1, 2, 3, 5, 5, 6, 9]), [1, 2, 3, 1])

    def test_renders_the_histogram(self) -> None:
        counts = [0, 2, 3, 4, 1]
        want = (
            "             usecs : count    distribution\n"
            "            0 -> 1 : 0        |                                        |\n"
            "            2 -> 3 : 2        |********************                    |\n"
            "            4 -> 7 : 3        |******************************          |\n"
            "           8 -> 15 : 4        |****************************************|\n"
            "          16 -> 31 : 1        |**********                              |"
        )
        self.assertEqual(render_histogram(counts, "usecs"), want)

    def test_renders_header_only_when_empty(self) -> None:
        self.assertEqual(
            render_histogram([0, 0, 0], "usecs"),
            "             usecs : count    distribution",
        )

    def test_trailing_empty_buckets_are_trimmed(self) -> None:
        out = render_histogram([3, 1, 0, 0], "nsecs")
        self.assertEqual(len(out.splitlines()), 3)  # header + 2 rows
        self.assertIn("2 -> 3 : 1", out)
        self.assertNotIn("4 -> 7", out)


if __name__ == "__main__":
    unittest.main()
