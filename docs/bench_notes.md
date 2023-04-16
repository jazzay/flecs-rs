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
simple_insert/hecs_single - 12.901 ms 
simple_insert/hecs_batched - 3.0754 ms
simple_iter/hecs - 91.187 µs

Flax 0.4.0
simple_insert/flax_single - 53.181 ms 
simple_insert/flax_batched - 1.6943 ms
simple_iter/flax - 97.714 µs	large gain! on par w Hecs now

flecs 3.0.1 (unchanged)
simple_insert/flecs - 69.735 ms
simple_iter/flecs_each - 722.49 µs 723.19 µs
simple_iter/flecs_iter - 35.810 µs


