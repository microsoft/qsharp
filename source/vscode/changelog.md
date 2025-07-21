# QDK Changelog

## v1.18.0

### What's Changed

- Log token budget and usage for Copilot tools by [@minestarks](https://github.com/minestarks) in [#2466](https://github.com/microsoft/qsharp/pull/2466)
- Added ApplyQPE and ApplyOperationPowerCA to Canon namespace by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2473](https://github.com/microsoft/qsharp/pull/2473)
- Fix bug in `Controlled SX` with empty controls by [@swernli](https://github.com/swernli) in [#2507](https://github.com/microsoft/qsharp/pull/2507)
- Support Running Projects from Circuit Files by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2455](https://github.com/microsoft/qsharp/pull/2455)
- Quantum Phase estimation sample via ApplyQPE by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2506](https://github.com/microsoft/qsharp/pull/2506)
- Circuit Editor Run Button by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2517](https://github.com/microsoft/qsharp/pull/2517)
- Don't show code lenses for code with compilation errors by [@copilot-swe-agent](https://github.com/copilot-swe-agent) in [#2511](https://github.com/microsoft/qsharp/pull/2511)
- Adding support for QASM 2.0 in the OpenQASM compiler by [@idavis](https://github.com/idavis) in [#2527](https://github.com/microsoft/qsharp/pull/2527)
- Fix language service to use Unrestricted target profile as default for notebooks by [@copilot-swe-agent](https://github.com/copilot-swe-agent) in [#2528](https://github.com/microsoft/qsharp/pull/2528)
- Generic resource estimation using Python models by [@msoeken](https://github.com/msoeken) in [#2555](https://github.com/microsoft/qsharp/pull/2555)

**Full Changelog**: [v1.17.0...v1.18.0](https://github.com/microsoft/qsharp/compare/v1.17.0...v1.18.0)

## v1.17.0

### OpenQASM support

We've added extensive support for the [OpenQASM](https://openqasm.com/) language. This provides editor support (syntax highlighting, intellisense, semantic errors), simulation, integration with Q#, and QIR code generation, amongst other features.

![image](https://github.com/user-attachments/assets/d6d78f6e-9dd1-4724-882b-a889d4ace4c8)

See the wiki page at <https://github.com/microsoft/qsharp/wiki/OpenQASM> for more details.

### Copilot improvements

We've improved the GitHub Copilot integration with this release. See the details at <https://github.com/microsoft/qsharp/wiki/Make-the-most-of-the-QDK-and-VS-Code-agent-mode>

### Circuit editor improvements

We have further improved the ability to edit circuit diagrams. See the detail at <https://github.com/microsoft/qsharp/wiki/Circuit-Editor>

### What's Changed

- Support intrinsic `SX` gate by [@swernli](https://github.com/swernli) in [#2338](https://github.com/microsoft/qsharp/pull/2338)
- Improved Drag and Drop by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2351](https://github.com/microsoft/qsharp/pull/2351)
- Support return values from custom intrinsics by [@swernli](https://github.com/swernli) in [#2350](https://github.com/microsoft/qsharp/pull/2350)
- Add copilot-instructions.md for our repo by [@minestarks](https://github.com/minestarks) in [#2365](https://github.com/microsoft/qsharp/pull/2365)
- Added tuple unpacking samples by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2381](https://github.com/microsoft/qsharp/pull/2381)
- Add/Remove Qubit Lines through Drag-and-Drop by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2372](https://github.com/microsoft/qsharp/pull/2372)
- Update Known Q# Tests Cases on QIR Profile Change by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2373](https://github.com/microsoft/qsharp/pull/2373)
- Fix bug with Circuit CSS not being applied to notebooks by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2395](https://github.com/microsoft/qsharp/pull/2395)
- Copilot tools for run, estimate, circuit by [@minestarks](https://github.com/minestarks) in [#2380](https://github.com/microsoft/qsharp/pull/2380)
- Break on `fail` during debugging by [@swernli](https://github.com/swernli) in [#2400](https://github.com/microsoft/qsharp/pull/2400)
- Add explicit cast support by [@orpuente-MS](https://github.com/orpuente-MS) in [#2377](https://github.com/microsoft/qsharp/pull/2377)
- Restore fancy error reporting in Python by [@swernli](https://github.com/swernli) in [#2410](https://github.com/microsoft/qsharp/pull/2410)
- OpenQASM Grover's algorithm sample by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2398](https://github.com/microsoft/qsharp/pull/2398)
- OpenQASM Bernstein-Vazirani sample by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2403](https://github.com/microsoft/qsharp/pull/2403)
- Added OpenQASM samples as templates in VSCode by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2416](https://github.com/microsoft/qsharp/pull/2416)
- Support Array Update Syntax by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2414](https://github.com/microsoft/qsharp/pull/2414)
- Needless operation lint should ignore lambdas by [@swernli](https://github.com/swernli) in [#2406](https://github.com/microsoft/qsharp/pull/2406)
- Added OpenQASM Ising sample by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2435](https://github.com/microsoft/qsharp/pull/2435)
- Adding copilot instructions by [@idavis](https://github.com/idavis) in [#2436](https://github.com/microsoft/qsharp/pull/2436)
- Fix explicit types in for loops by [@swernli](https://github.com/swernli) in [#2440](https://github.com/microsoft/qsharp/pull/2440)
- Check that non-void functions always return by [@orpuente-MS](https://github.com/orpuente-MS) in [#2434](https://github.com/microsoft/qsharp/pull/2434)
- Added OpenQASM simple teleportation sample by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2441](https://github.com/microsoft/qsharp/pull/2441)
- Add sample Python integration and resource estimation notebooks for OpenQASM by [@idavis](https://github.com/idavis) in [#2437](https://github.com/microsoft/qsharp/pull/2437)
- Fix panic when passing wrong literal kind as modifier arg by [@orpuente-MS](https://github.com/orpuente-MS) in [#2446](https://github.com/microsoft/qsharp/pull/2446)
- Fix bit shifts with bit literals on lhs by [@orpuente-MS](https://github.com/orpuente-MS) in [#2450](https://github.com/microsoft/qsharp/pull/2450)
- Fix panic due to missing `Unit` value from assignment by [@swernli](https://github.com/swernli) in [#2452](https://github.com/microsoft/qsharp/pull/2452)
- Create copilot-setup-steps.yml by [@minestarks](https://github.com/minestarks) in [#2445](https://github.com/microsoft/qsharp/pull/2445)
- Added OpenQASM samples into the VSCode Playground by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2458](https://github.com/microsoft/qsharp/pull/2458)

**Full Changelog**: [v1.16.0...v1.17.0](https://github.com/microsoft/qsharp/compare/v1.16.0...v1.17.0)

## v1.16.0

### Copilot integration

With VS Code Copilot integration you can now use Copilot to to assist with many tasks such as writing code, generating tests, connecting to an Azure Quantum workspace, submit jobs to run on hardware, and more!

<img width="547" alt="image" src="https://github.com/user-attachments/assets/f417ef8f-be4c-4ae5-9c0e-c0c18b3e7021" />

See the wiki page at <https://github.com/microsoft/qsharp/wiki/Make-the-most-of-the-QDK-and-VS-Code-agent-mode> for more info, as well as tips and best practices.

### Circuit Editor

You can now add .qsc files to your project which provide a drag-and-drop circuit editor user interface to create quantum operations, which can then be called from your Q# code.

<img width="1133" alt="image" src="https://github.com/user-attachments/assets/d4e492ab-8232-4392-908d-f0a6f9b8d45b" />

See the wiki page at <https://github.com/microsoft/qsharp/wiki/Circuit-Editor> for more details.

### What's Changed

- Fix Test Explorer issues by [@billti](https://github.com/billti) in [#2291](https://github.com/microsoft/qsharp/pull/2291)
- Circuit Editor by [@ScottCarda-MS](https://github.com/ScottCarda-MS) in [#2238](https://github.com/microsoft/qsharp/pull/2238)
- Add lint groups to Q# by [@orpuente-MS](https://github.com/orpuente-MS) in [#2103](https://github.com/microsoft/qsharp/pull/2103)
- Added RoundHalfAwayFromZero to standard library by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2321](https://github.com/microsoft/qsharp/pull/2321)
- Added BigIntAsInt to Std.Convert by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2325](https://github.com/microsoft/qsharp/pull/2325)
- Added ApplyOperationPowerA to Std.Canon by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2324](https://github.com/microsoft/qsharp/pull/2324)
- Add Python evaluation API by [@idavis](https://github.com/idavis) in [#2345](https://github.com/microsoft/qsharp/pull/2345)
- Add "Update Copilot instructions" command by [@minestarks](https://github.com/minestarks) in [#2343](https://github.com/microsoft/qsharp/pull/2343)
- Add Ising model samples by [@DmitryVasilevsky](https://github.com/DmitryVasilevsky) in [#2342](https://github.com/microsoft/qsharp/pull/2342)
- Add GitHub Copilot tools for Azure Quantum by [@minestarks](https://github.com/minestarks) in [#2349](https://github.com/microsoft/qsharp/pull/2349)

**Full Changelog**: [v1.15.0...v1.16.0](https://github.com/microsoft/qsharp/compare/v1.15.0...v1.16.0)
