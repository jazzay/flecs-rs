# Notes

Right now the benches are super basic, but at least we can get a sense of relative performance between the various ECSs covered. And it's useful to see how Flecs versions progress.

# Bench history on M1 Max (Dec 2022)

Bevy 0.9.1:
simple_insert/bevy_single - 18.307 ms
simple_insert/bevy_batched - 3.1692 ms
simple_iter/bevy - 120.06 µs

Hecs 0.9.0
simple_insert/hecs_single - 10.207 ms 
simple_insert/hecs_batched - 2.9633 ms
simple_iter/hecs - 91.739 µs

Flax 0.3.2
simple_insert/flax_single - 51.808 ms 
simple_insert/flax_batched - 1.5515 ms
simple_iter/flax - 237.27 µs

flecs 3.0.1
simple_insert/flecs - 69.735 ms
simple_iter/flecs_each - 722.49 µs 723.19 µs
simple_iter/flecs_iter - 35.810 µs

# Bench history on M1 Max (Apr 2023)

Bevy 0.10.3:
simple_insert/bevy_single - 14.718 ms
simple_insert/bevy_batched - 2.9090 ms
simple_iter/bevy - 122.76 µs

Hecs 0.10.1
simple_insert/hecs_single - 10.123 ms 
simple_insert/hecs_batched - 2.8848 ms
simple_iter/hecs - 91.187 µs

Flax 0.4.0
simple_insert/flax_single - 52.398 ms 
simple_insert/flax_batched - 1.5866 ms
simple_iter/flax - 97.714 µs	large gain! on par w Hecs now

flecs 3.0.1 (unchanged)
simple_insert/flecs - 69.735 ms
simple_iter/flecs_each - 722.49 µs
simple_iter/flecs_iter - 35.810 µs

# Flecs 3.1.0 Release (Apr 2023)
Other 3 similar to above

flecs 3.1.0
simple_insert/flecs - 70.977 ms
simple_iter/flecs_each - 724.53
simple_iter/flecs_iter - 35.739 µs

# Flecs 3.1.2 Release (Apr 2023)
Other 3 similar to above
Similar results, slightly worse

flecs 3.1.2
simple_insert/flecs - 75.482 ms
simple_iter/flecs_each - 741.34 µs
simple_iter/flecs_iter - 36.590 µs

# Flecs 3.1.3 Release (Apr 2023)
Pretty much the same results
Should add better benches once 3.2 upgrade is completed.

# Flecs 3.1.4 Release (Apr 2023)
flecs 3.1.2
simple_insert/flecs - 59.603 ms		20% savings!
simple_iter/flecs_each - 732.62 µs
simple_iter/flecs_iter - 35.545 µs

# Flecs 3.1.5 Release (Apr 2023)
Pretty much the same results as for 3.1.4


