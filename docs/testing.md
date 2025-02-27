# Testing in `wgpu` and `naga`

The `wgpu` repository has a broad and deep set of APIs that are so complex that
testing is a necessity for doing work.

## Testing against your own application(s)

When testing your own code against some changes, we recommend [using a `path`
dependency][path-deps] in Cargo for local testing of changes, and a [`git`
dependency][git-deps] pointing to your own fork to share changes with other
contributors.

[path-deps]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies
[git-deps]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories

## An Overview of Tests in `wgpu`

This is a table of contents, in the form of the repository's directory structure.

- benches
  - [benches](#benchmark-tests)
- examples
  - [features](#example-tests)
- naga
  - tests
    - [example_wgsl](#naga-example-tests)
    - [snapshot](#naga-snapshot-tests)
    - [spirv-capabilities](#naga-spirv-capabilities-test)
    - [validation](#naga-validation)
    - [wgsl_errors](#naga-wgsl-error-tests)
- player
  - [tests](#player-tests)
- tests
  - [compile](#wgpu-compile-tests)
  - [dependency](#wgpu-dependency-tests)
  - [gpu](#wgpu-gpu-tests)
  - [validation](#wgpu-validation-tests)

Where applicable, [unit tests](#unit-tests) are scattered throughout the
codebase.

## Continuous Integration Tests

Automated testing is the best way to ensure we aren't unduly breaking users as
we develop it. This section serves as a guide to the large variety of such tests
that the `wgpu` repository has, describing each class of test and what they are
used for.

### Requirements

- The [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) should be installed on
  the system and the `bin` folder of the SDK is in your `PATH`. Without this,
  some tests may fail to run, or report false negatives.
- [`cargo-nextest`](https://nexte.st/#quick-start) is used to drive automated
  tests. Generally, this is called by bespoke `xtask` automation in the
  repository.

### Run All Tests

To run all automated tests, run `cargo xtask test` from the root of the
repository.

### Benchmark Tests

- Located in: `benches/benches`
- Run with `cargo nextest run --bench wgpu-benchmark`
- `wgpu` benchmarks for performance testing.

These are benchmarks that test the performance of `wgpu` in various
scenarios. When run as part of the test suite, they run a single
iteration of each benchmark to ensure they continue to function.

These tests only run on your system's default GPU.

The benchmarks should be very careful to avoid doing any significant
work (including connecting to a GPU) outside of the various `benchmark`
`criterion` functions. If this is done, the benchmarks will take a long
time to list available tests, slowing down the test suite.

To run the benchmarks for benchmarking purposes, use `cargo bench`.

### Example Tests

- Located in: `examples/features`
- Run with `cargo xtask test --bin wgpu-examples`
- Uses a custom `#[gpu_test]` harness.
- `wgpu` integration tests, with access to `wgpu_test` helpers.

These tests validate that the examples are functioning correctly
and do not have any regressions. They use the same harness as the
[gpu tests](#wgpu-gpu-tests), see that section for more information
on the harness.

These tests use `nv-flip`'s image comparison through the wgpu
example framework to validate that the images outputted by the
examples are within tolerance of the expected output.

Examples written in `examples/standalone` do not have tests, as
they should be easy to copy into a standalone project.

### `naga` Example Tests

- Located in: `naga/tests/naga/example_wgsl`
- Run with `cargo nextest run --test naga example_wgsl`

This simple test ensures that all wgsl files in the `examples`
directory can be parsed by `naga`'s `wgsl` parser and validate correctly.

### `naga` Snapshot Tests

- Located in: `naga/tests/naga/snapshot`, `naga/tests/in`, and `naga/tests/out`
- Run with `cargo nextest run --test naga snapshots`
- Data driven snapshot tests for `naga`'s input/output.

These tests are snapshot tests for `naga`s parsers and code generators.
There are inputs in `wgsl`, `spirv`, and `glsl`. There are outputs for
`hlsl`, `spirv`, `wgsl`, `msl`, `glsl`, and naga's internal IR. The tests
can be configured by a sidecar toml file of the same name as the input file.

This is the goto tool for testing all kinds of codegen and parsing features.

To avoid clutter, we generally use the following pattern:

- `wgsl` tests generate output to all backends.
- The `spirv` and `glsl` tests generate `wgsl` output.

This "butterfly" pattern ensures we don't need to test the
full matrix of possibilities to get full coverage.

While we do not run the results of the code generators, we do
test that the generated code is valid. This is done by running
`cargo xtask validate <backend>` in the `naga` directory and
will use the respective tool to validate the generated code.

### `naga` SPIR-V Capabilities Tests

- Located in: `naga/tests/naga/spirv_capabilities`
- Run with `cargo nextest run --test naga spirv_capabilities`
- Uses the standard `#[test]` harness.

These tests convert the given wgsl snippet to spirv and
then assert that the spirv has enabled the expected capabilities.

### `naga` Validation Tests

- Located in: `naga/tests/naga/validation`
- Run with `cargo nextest run --test naga validation`

These are hand rolled tests against the naga's validator.
If you don't need to test the validator with a custom module,
and can use the `wgsl` frontend, you should put the test in
the [wgsl errors](#naga-wgsl-error-tests) tests.

### `naga` WGSL Error Tests

- Located in: `naga/tests/naga/wgsl_errors`
- Run with `cargo nextest run --test naga wgsl_errors`

These are tests for the error messages that the `wgsl` frontend
produces. Additionally you can check that a given validation error
is produced by the validator from a given `wgsl` snippet.

### `player` Tests

- Located in: `player/tests`
- Run with `cargo nextest run --test player`
- Data driven tests using the `player`'s replay system.
- `wgpu` integration tests.

These are soft-deprecated tests which are another way to write
API tests. These use captures of the api calls and replay them
to assert on the behavior. They are very difficult to write, and
the trace capturing system is currently broken, so these
tests exist, but you should not write new ones.

These tests only run on your system's default GPU.

### `wgpu` Compile Tests

- Located in: `tests/tests/wgpu-compile`
- Run with `cargo nextest run --test wgpu-compile`
- `trybuild` tests of all rust files in `tests/tests/wgpu-compile/fail` directory.

These use the `trybuild` crate to test a few scenarios where
the `wgpu` crate is expected to fail to compile. This mainly
revolves around ensuring lifetimes are properly handled when
dropping passes, etc.

### `wgpu` Dependency Tests

- Located in: `tests/tests/wgpu-dependency`
- Run with `cargo nextest run --test wgpu-dependency`
- Tests against `cargo tree`.

These tests ensure that the `wgpu` crate has the correct dependency
tree on all platforms. It's super easy to subtly mess up the dependencies
which can cause issues or extra dependencies to be pulled in.

This provides a way to ensure that our `toml` files are correct.

### `wgpu` GPU Tests

- Located in: `tests/tests/wgpu-gpu`
- Run with `cargo xtask test --test wgpu-gpu`
- Uses a custom `#[gpu_test]` harness.
- `wgpu` integration tests, with access to `wgpu_test` helpers.

These tests use a custom harness to run each test on all GPUs
available on the system. They are general integration tests
that write code against the normal `wgpu` API and assert on the behavior.

These tests are useful to check the runtime behavior of a program,
validate that there are no validation errors coming from the
`vulkan`/`dx12`/`metal` validation layers, and ensure behavior
is the same across GPUs. If the test does not need to run on a
real GPU, it should be in the [validation tests](#wgpu-validation-tests) instead.

There is a special parameter system that deals with if a GPU
can support the given test, and dealing with expectation
management for tests that are expected to fail due to driver or wgpu bugs.

Normal `#[test]`s will not be found in this test crate, as we use a custom harness.

See also the [example tests](#example-tests) for additional GPU tests.

### `wgpu` Validation Tests

- Located in: `tests/tests/wgpu-validation`
- Run with `cargo nextest run --test wgpu-validation`
- Use the standard `#[test]` harness.
- `wgpu` integration tests, with access to `wgpu_test` helpers.

These tests are focused on testing the validation inside of `wgpu-core`.
They are written against the `wgpu` API, but are targeting a special `noop`
backend which does not connect to a real GPU.

This is significantly faster and simpler than running on real hardware,
and allows any validation logic to be checked, even if real hardware
does not support those features.

### Unit Tests

- Located throughout the codebase.
- Run with `cargo nextest test -p <package>`
- Standard `#[test]`s.

Throughout the codebase we have standard `#[test]`s that test individual
functions or small parts of the codebase. These don't run on the gpu.
