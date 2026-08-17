Dump from Claude that I haven't squared against my understanding yet for
actually implementing `subgroup-size-control`:

naga PR (land + vendor first):

1. ir/mod.rs:2522 — add required_subgroup_size: Option<u32> to EntryPoint. (No compact/ change.)
2. enable_extension.rs — promote SubgroupSizeControl from Unimplemented → Implemented (enum, VARIANTS, from_ident/to_ident, add(), capability() → Caps::SUBGROUP_SIZE_CONTROL); kill the todo!() at :341.
3. parse/mod.rs:2016 — replace the reject-stub with real parse (require_enable_extension, ( expr ), accumulator by :1851); AST field ast.rs:216; construct mod.rs:2181; lower lower/mod.rs:1744.
4. valid/interface.rs:1280 — gate on Caps::SUBGROUP_SIZE_CONTROL, require compute, power-of-two/range check; new EntryPointError at :130.
5. Backends: SPIR-V ExecutionMode::SubgroupSize at back/spv/writer.rs:1865 (+require_any cap/SPV_EXT_subgroup_size_control); HLSL [WaveSize(N)] at back/hlsl/writer.rs:548 gated SM6.6+.
6. Snapshots + wgsl_errors.rs cases.

wgpu PR (after vendor): 7. wgpu-naga-bridge/src/lib.rs:126 — map the feature → Caps::SUBGROUP_SIZE_CONTROL. 8. wgpu-core/src/validation.rs:341 + :1422 — carry required_subgroup_size into validation::EntryPoint; validate in check_stage (:1689) against AdapterInfo.subgroup_min/max_size + feature-enabled. 9. Vulkan adapter.rs:897 — set F::SUBGROUP_SIZE_CONTROL (ext-supported && SUBGROUP); :1318 — gate the pre-1.3 ext push on the new feature. DX12 adapter.rs:594 — set the feature when SM6.6/DXC + waveops. (No HAL descriptor field under Option A.) 10. Decide SUBGROUP implication (auto-add vs. error); review limits.rs:169 buckets/exempt. 11. Feature doc features.rs:1852 (Vulkan+DX12, Metal unsupported), gpu integration test, CHANGELOG.

Two gotchas to keep in your pocket while implementing: (a) the SPIR-V execution mode still requires the Vulkan subgroupSizeControl device feature to be enabled — it already is (adapter.rs:541), so nothing to add there; (b) @subgroup_size(N) must be a power of two within [subgroup_min_size, subgroup_max_size], and that range is per-adapter — validate against the actual AdapterInfo values, not the 4..128 constants (those are only fallbacks).

Ping me when you hit a specific layer and want a hand.
