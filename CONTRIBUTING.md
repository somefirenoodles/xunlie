# Contribuir a Xunlie

## Antes de abrir una PR

1. Vincula un issue y al menos un `REQ-*` o explica por qué es mantenimiento sin cambio de requisito.
2. Satisface la Definition of Ready de `docs/process/DEVELOPMENT-LIFECYCLE.md`.
3. Ejecuta `python scripts/validate_quality_system.py`; cuando exista código, también `cargo xtask quality`.
4. Incluye tests, documentación y trazabilidad en el mismo cambio.
5. Declara impacto de arquitectura, seguridad, compatibilidad y uso material de IA.

No añadas una dependencia de producción, cambies un protocolo/esquema o una frontera sin ADR aprobado.

## Revisión

El autor no aprueba su propia PR. Cambios críticos requieren dos revisores y CODEOWNER. Un check verde no sustituye revisión de intención.

## Reportes de seguridad

No publiques vulnerabilidades explotables en issues. Usa el canal descrito en `SECURITY.md`.

