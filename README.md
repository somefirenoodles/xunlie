# Xunlie

Xunlie, anteriormente **InvariantCI**, es un sistema de aseguramiento para ingeniería de software agéntica. Compila fuentes de requisitos en un contrato verificable, genera historias contractualmente equivalentes, ejecuta agentes en entornos aislados y bloquea una entrega cuando el resultado depende indebidamente del camino de especificación.

Este repositorio comienza por su sistema de calidad. La regla es deliberada: ningún incremento de producto se considera terminado si no conserva arquitectura, responsabilidades, trazabilidad requisito-función y evidencia reproducible.

## Estado

- Nombre del producto: `xunlie`.
- Estado de la baseline: propuesta para aprobación (`0.1.0`).
- Código de producto: aún no iniciado.
- Calidad como código: activa para documentación, arquitectura y trazabilidad.
- Publicación objetivo: repositorio público `somefirenoodles/xunlie` bajo licencia MIT.

## Mapa del repositorio

| Ruta | Propósito |
|---|---|
| `docs/architecture/` | Baseline, límites, dependencias y ADR |
| `docs/quality/` | Software Quality Plan, gates, métricas y auditoría |
| `docs/process/` | Ciclo de desarrollo, roles y control de cambios |
| `docs/requirements/` | Requisitos funcionales y de calidad |
| `quality/` | Contratos ejecutables del sistema de calidad |
| `scripts/` | Validadores sin dependencias externas |
| `.github/` | Gobernanza y automatización para el repositorio remoto |

## Comprobación local

```powershell
python scripts/validate_quality_system.py
```

Un resultado correcto termina con `QUALITY SYSTEM: PASS`. Esto no certifica el producto; certifica que la baseline de gobierno es internamente consistente y que toda relación obligatoria está declarada.

## Principio de aceptación

La puntuación de una etapa solo puede ser `100/100` si todos sus controles obligatorios pasan y toda la evidencia está presente. Un único control crítico fallido produce `BLOCKED`, aunque el promedio numérico sea alto.

## Documentos normativos internos

1. [Software Quality Plan](docs/quality/SOFTWARE-QUALITY-PLAN.md)
2. [Arquitectura](docs/architecture/ARCHITECTURE.md)
3. [Gates de etapa](docs/quality/STAGE-GATES.md)
4. [Ciclo de desarrollo](docs/process/DEVELOPMENT-LIFECYCLE.md)
5. [Requisitos](docs/requirements/REQUIREMENTS.md)
6. [Decisiones abiertas](docs/decisions/OPEN-DECISIONS.md)
