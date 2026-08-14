# Endpoint definitions

- `first_fission_by_horizon`: founder has a real `FISSION_COMPLETED` event by
  the frozen horizon.
- `time_to_first_qualified_fission`: accepted simulated time of that event.
- `restricted_reproduction_time`: first-fission time, or `80.0` when there is
  no fission. Death before reproduction also maps to `80.0`.
- `founder_death_before_reproduction`: founder death event occurred before any
  founder fission.
- `live_gen1_daughter_count_at_first_fission`: living generation-1 records at
  the first fission; zero when no fission occurs.
- `accepted_steps_at_stop` and `accepted_simulated_time_at_stop`: direct
  harness stop values.

Fission incidence is reported without conditioning away non-reproducing
founders. The raw per-replicate records are in `replicate_endpoints.json`.

