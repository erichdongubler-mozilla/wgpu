---
name: cts-triage
description: Run CTS test suites and investigate failures
---

# CTS Query Anatomy

Every selector in this skill is a CTS query. Get the syntax right before running
anything; a malformed query is a hard parse error, not a zero-match run.

```
webgpu:api,validation,buffer,create:usage:usage1=0;usage2=0
```

| level | component   | value in example               | where it comes from                                   |
| ----- | ----------- | ------------------------------ | ----------------------------------------------------- |
| 1     | suite       | `webgpu`                       | `cts/src/webgpu/`                                     |
| 2     | file path   | `api,validation,buffer,create` | `cts/src/webgpu/api/validation/buffer/create.spec.ts` |
| 3     | test path   | `usage`                        | `g.test('usage')` in that file                        |
| 4     | case params | `usage1=0;usage2=0`            | `.params(…)` on that test                             |

```
query      := suite ":" filePath ":" testPath ":" caseParams
filePath   := part ("," part)*   # directories; last part is the .spec.ts basename
testPath   := part ("," part)*   # the literal string passed to g.test()
caseParams := kv (";" kv)*
kv         := key "=" jsonValue
part, key  := /^[a-zA-Z0-9_]+$/
```

- `:` divides the four levels. `,` divides parts _within_ the file path or test
  path. `;` divides case params. `=` divides a param key from its value.
- Derivation: `<suite>:<a>,<b>,<c>` → `cts/src/<suite>/<a>/<b>/<c>.spec.ts`.
- Param values are JSON, so strings are quoted: `format="stencil8"`.

## Truncating a query with `*`

A query may stop at any level to match many tests, but `*` must be the
**complete final part**. `foo*` and `*,foo` are errors.

| query                                                         | matches                       |
| ------------------------------------------------------------- | ----------------------------- |
| `webgpu:api,validation,*`                                     | every file under that dir     |
| `webgpu:api,validation,buffer,create:*`                       | every test in that file       |
| `webgpu:api,validation,buffer,create:usage:*`                 | every case of that test       |
| `webgpu:api,validation,buffer,create:usage:usage1=0;*`        | cases matching a param prefix |
| `webgpu:api,validation,buffer,create:usage:usage1=0;usage2=0` | exactly one case              |

Omitting the trailing `*` at level 2 or 3 is an **error**, not an implicit
wildcard. CTS's parse error names both candidate fixes; pick the level you
actually meant.

## Mistakes to avoid

- **Level 3 is a path, not a name.** Test paths routinely contain commas:
  `g.test('mapAsync,state,mapped')` becomes
  `webgpu:api,validation,buffer,mapping:mapAsync,state,mapped:*`.
- **`,*` and `:*` are not interchangeable.** If one file contains tests `test`,
  `test,foo`, and `test,bar`, then `test,*` matches all three, while `test:*`
  matches only `test`. Prefer `,*` when collapsing selectors.
- **`:` cannot appear inside a suite, file path, or test path.** It is reserved
  as the level divider, and path parts are restricted to `[a-zA-Z0-9_]+`.
- **Subcase parameters are not addressable.** The narrowest query CTS can
  express is a single case, which always runs every subcase it contains. You
  cannot select, skip, or list an individual subcase.

# Triage Process

When working on a category of CTS tests, follow this systematic process to identify issues, prioritize fixes, and document findings.

## Step 0: Divide Into Manageable Chunks

List all the tests matching a selector:

```bash
cargo xtask cts -- --list 'webgpu:api,validation,*' 2>&1 | wc -l
```

If there is a reasonable number of tests matching the selector (less than a
couple hundred or so, but this isn't a hard cutoff), then you can proceed with
triage. Otherwise, make a list of more detailed wildcards that match fewer
tests, verify that each wildcard matches a reasonable number of tests, and
triage each wildcard separately.

## Step 1: Get Overall Statistics

Run the full test suite for the category to understand the scope:

```bash
cargo xtask cts 'webgpu:api,validation,category:*' 2>&1 | grep -E "(Summary|Passed|Failed)" | tail -5
```

This gives you the pass rate and number of failures. Document this as your baseline.

## Step 2: Identify Test Subcategories

Review the output from running the CTS (with or without `--list`) to identify
any subcategories that may exist within the suite being analyzed. Subcategories
show up in one of two places:

- **As extra `,`-delimited parts of the test path** (level 3) — e.g. the tests
  `mapAsync,read,typedArrayAccess` and `mapAsync,write,typedArrayAccess` share
  the `mapAsync` subcategory, selectable as `…:mapAsync,*`.
- **As case parameters** (level 4) — e.g. `isAsync=false;*` or `format="*"`-ish
  groupings, selectable as a param prefix like `…:usage:isAsync=false;*`.

Never use `:` to reach a subcategory; `:` only ever divides the four levels.

Running tests by subcategory is usually more manageable than running the whole
suite at once or running individual tests.

## Step 3: Run Each Subcategory

Test each subcategory individually to identify which ones are passing vs failing:

```bash
cargo xtask cts 'webgpu:api,validation,category:subcategory:*' 2>&1 | grep -E "(Passed|Failed|Skipped)" | tail -3
```

Track the pass rate for each subcategory. This helps you identify:

- What's already working (don't break it!)
- Where the failures are concentrated
- Which issues affect multiple categories

## Step 4: Analyze Failure Patterns

For failing subcategories, look at what tests are failing:

```bash
cargo xtask cts 'webgpu:api,validation,category:subcategory:*' 2>&1 | grep "\[fail\]" | head -20
```

Look for patterns:

- **Format-specific failures**: May indicate missing format capability checks
- **Parameter-specific failures**: Validation missing for specific parameter combinations

## Step 5: Examine Specific Failures

Pick a representative failing test and run it individually to see the error.
Copy the selector verbatim from the `[fail]` line — do not hand-assemble it, and
remember that a test-level selector needs a trailing `:*`:

```bash
# All cases of one test (note the `:*`):
cargo xtask cts 'webgpu:api,validation,category:subcategory,specific_test:*' 2>&1 | tail -30

# One exact case, params included:
cargo xtask cts 'webgpu:api,validation,category:subcategory,specific_test:foo=false;bar="value"' 2>&1 | tail -30
```

Dropping the `:*` from the first form makes CTS read `specific_test` as a case
parameter and fail with `Param in a query must be of form key=value`.

Look for:

- **"EXPECTATION FAILED: DID NOT REJECT"**: wgpu is accepting invalid input (validation gap)
- **"Validation succeeded unexpectedly"**: Similar to above
- **"Unexpected validation error occurred"**: wgpu is rejecting valid input
- **Error message content**: Tells you what validation is triggering or missing

## Step 6: Check CTS Test Source

To understand what the test expects, read the TypeScript source:

```bash
grep -A 40 "g\.test('test_name'" cts/src/webgpu/api/validation/path/file.spec.ts
```

Anchor the search on `g.test('…'` rather than the bare name: test paths are
substrings of each other (`mapAsync` matches `mapAsync,state,mapped`), so a
bare pattern will land you in the wrong test.

The test source shows:

- What configurations are being tested
- What the expected behavior is (pass/fail)
- The validation rules from the WebGPU spec
- Comments explaining the rationale

## Step 7: Categorize Issues

Group failures into categories:

**High Priority - Validation Gaps:**

- wgpu accepts invalid configurations that should fail
- Security or correctness implications
- Example: Accepting wrong texture formats, missing aspect checks

**Medium Priority - Spec Compliance:**

- Edge cases not handled correctly
- Optional field validation issues
- Example: depthCompare optional field handling

**Low Priority - Minor Gaps:**

- Less common scenarios
- Limited real-world impact
- Example: Depth bias with non-triangle topologies

**Known Issues - Skip:**

- Known failure patterns (documented in Common Patterns below)
- Track count but don't try to fix

## Step 8: Identify Root Causes

For validation gaps, find where validation should happen:

1. **Search for existing validation:**

   ```bash
   grep -n "relevant_keyword" wgpu-core/src/device/resource.rs
   ```

2. **Look for render/compute pipeline creation:**
   - Render pipeline: `wgpu-core/src/device/resource.rs` around `create_render_pipeline`
   - Compute pipeline: Similar location
   - Look for existing validation patterns you can follow

3. **Check for helper functions:**

   ```bash
   grep "fn is_" wgpu-types/src/texture/format.rs
   ```

4. **Find error enums:**
   ```bash
   grep "pub enum.*Error" wgpu-core/src/pipeline.rs
   ```

## Step 9: Implement Fixes

When implementing fixes:

1. **Add error variants if needed** (in `wgpu-core/src/pipeline.rs`)
2. **Add helper methods** (in `wgpu-types` if checking properties)
3. **Add validation checks** (in `wgpu-core/src/device/resource.rs`)
4. **Test the fix** with specific failing tests
5. **Run full subcategory** to verify all related tests pass
6. **Check you didn't break passing tests**

## Step 10: Document Findings

Create or update a triage document (e.g., `category_triage.md`).

Do not write information about changes you have made to the triage document. Only capture the state of the tests and any investigation into open issues.

````markdown
# Category CTS Tests - Triage Report

**Overall Status:** XP/YF/ZS (%/%/%)

## Passing Sub-suites ✅

[List sub-suites that have no failures (all pass or skip)]

## Remaining Issues ⚠️

[List sub-suites that have failures and if it can be stated concisely, a summary of the issue]

## Issue Detail

[List detail of any investigation into failures. Do not go into detail about passed suites, just list the failures.]

### 1. title, e.g. a distinguishing word from the test selector

**Test selector:** `webgpu:api,validation,render_pipeline,depth_stencil_state:format:*`
**What it tests:** [Description]
**Example failure:**
[a selector for a single failing test, e.g.:]

```
webgpu:api,validation,render_pipeline,depth_stencil_state:depthCompare_optional:isAsync=false;format="stencil8"
```

**Error:**
[error message from the failing tests, e.g.:]

```
Unexpected validation error occurred: Depth/stencil state is invalid:
Format Stencil8 does not have a depth aspect, but depth test/write is enabled
```

**Root cause:**
[Your analysis of the root cause. Do not speculate. Only include the results of specific investigation you have done.]
The validation is triggering incorrectly. When `depthCompare` is undefined/missing in JavaScript, it's getting translated to a default value that makes `is_depth_enabled()` return true, even for stencil-only formats.

**Fix needed:**
[Your proposed fix. Again, do not speculate. Only state the fix if it is obvious from the root cause analysis, or if you have done specific investigation into how to fix it.]

### 2. title

[repeat as needed for additional issues]
````

## Step 11: Update CTS Result Lists

The result lists have these purposes:

- `cts_runner/test.lst` contains sets of tests with no failures. It is not an
  exhaustive list of passing sets of tests.
- `cts_runner/fail.lst` contains sets of tests with failures. A comment can record
  the pass rate, especially when the pass rate is high.
- `cts_runner/skip.lst` contains sets of tests in which at least 90% of the tests
  are skipped.

If you fix a set of tests, add it to `cts_runner/test.lst` only when all of its
tests pass or skip. Do not add a set of tests that has any failures.

Use wildcards to minimize the number of selectors:

```
webgpu:api,validation,category:subcategory:isAsync=false;*
```

Use a higher-level wildcard only when all tests that it matches belong in the
same result list. Do not use a wildcard that also matches tests from another
result list.

Keep the selectors in the order that the `lst_files_are_sorted` integration
test requires. This rule also applies to selectors in comments.

## Step 12: Verify and Build

Before finishing:

```bash
# Format code
cargo fmt

# Check for errors
cargo clippy --tests

# Build to ensure no compilation errors
cargo build

# Run the tests you added to test.lst
cargo xtask cts 'webgpu:api,validation,category:subcategory:isAsync=false;*'
```

# Common Patterns

If the user asked to investigate a failure, and the cause of the failure is
noted here with "do not attempt to fix", then stop and ask the user before
attempting to fix.

**Pattern: Format-specific failures**

- Check if format validation is missing
- Look for `is_depth_stencil_format()`, `is_color_format()` etc.
- May need to add format capability checks

**Pattern: Aspect-related failures**

- Check if code validates format aspects (DEPTH, STENCIL, COLOR)
- Use `hal::FormatAspects::from(format)` to check
- Validate operations match available aspects

**Pattern: Optional field failures**

- May be WebGPU optional field semantics issue
- Check if undefined in JS becomes a default value in Rust
- May need to distinguish "not set" from "set to default"

**Pattern: Atomics accepted incorrectly**

- Naga allows referencing an atomic directly in an expression
- Should only allow accessing via `atomicLoad`, `atomicStore`, etc.
- Only investigate as necessary to confirm this is the issue. Do not attempt to fix. Refer user to https://github.com/gfx-rs/wgpu/issues/5474.

**Pattern: Error reporting for destroyed resources**

- Tests that check for validation errors when a destroyed resource is used. `wgpu` often reports these errors later than WebGPU requires, causing the tests to fail.
- `wgpu` may report these errors earlier than it should, causing the test to fail with an unexpected validation error.
- Look for:
  - Tests with `state="destroyed"` parameter
  - Tests checking that operations on destroyed buffers or textures should fail
- Example failing tests:
  - `webgpu:api,validation,encoding,cmds,compute_pass:indirect_dispatch_buffer_state:` with `state="destroyed"` subcases
- Only investigate as necessary to confirm this is the issue. Do not attempt to fix. Refer user to https://github.com/gfx-rs/wgpu/issues/7881.

# Tips

- **Start with high-impact fixes**: Validation gaps with security implications
- **Look for existing patterns**: Other validation code shows the style
- **Test incrementally**: Fix one category at a time, verify it works
- **Document as you go**: Don't wait until the end to write the triage
- **Ask for clarification**: If test expectations are unclear, check the WebGPU spec or test source
- **Track your progress**: Update pass rates as you fix issues
