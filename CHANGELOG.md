# Changelog

Los cambios relevantes de Xunlie se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto
usa [versionado semántico](https://semver.org/lang/es/).

## [Unreleased]

### Added

- M2 certified history variants with executable preconditions, deterministic replay, and
  `xunlie.equivalence-certificate/v1` evidence.
- Built-in JSON-normalization and independent-add reversal operators.
- CLI `variant` and `verify-variant` commands with stable exclusion and verification exit codes.
- Golden, metamorphic, tamper, exclusion, and directed mutation testing for variant logic.

## [0.1.0] - 2026-08-13

### Added

- Compilador determinista desde fuentes `xunlie.source/v1` a `ContractIR`.
- Operaciones de historial `add`, `replace` y `revoke` con conflictos estructurados.
- Digests semántico y de artefacto para contenido, procedencia y metadatos.
- CLI `xunlie compile` y `xunlie validate` con salida humana o JSON.
- Suite de pruebas, controles de arquitectura y puerta local `cargo xtask quality`.
- CI para Linux y Windows, auditoría de dependencias y análisis CodeQL.
- Jobs bloqueantes de MSRV, cobertura mínima y fuzzing del parser con corpus versionado.
- Releases binarios para Linux/Windows con checksums y build provenance.
- Guías de contribución, seguridad, conducta y plantillas de colaboración pública.

### Security

- Fallo cerrado ante una fuente inválida o un conflicto no resuelto, sin emitir un contrato
  parcial.
- Reporte privado de vulnerabilidades y threat model documentado.

[Unreleased]: https://github.com/somefirenoodles/xunlie/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/somefirenoodles/xunlie/releases/tag/v0.1.0
