# Control de cambios y evolución tecnológica

## Clases

| Clase | Ejemplo | Aprobación mínima |
|---|---|---|
| C0 editorial | typo sin cambio semántico | 1 reviewer |
| C1 compatible | función aditiva interna | codeowner + Quality |
| C2 arquitectura/API | frontera, esquema, protocolo, dependencia prod | ADR + Architect + Quality + Security si aplica |
| C3 ruptura/riesgo alto | breaking change, aislamiento, criptografía, release | PO + Architect + Quality + Security |
| C4 emergencia | incidente activo | break-glass + revisión en 24 h |

## Solicitud

Toda solicitud registra motivación, alternativas, requisitos, impacto, riesgos, compatibilidad, migración, pruebas, observabilidad y rollback. La clase puede elevarse durante revisión.

## Adopción de herramienta o técnica state-of-the-art

1. **Radar:** capturar problema y fuente primaria; no adoptar todavía.
2. **Hipótesis:** declarar mejora medible y baseline comparable.
3. **Sandbox:** evaluar precisión, determinismo, seguridad, licencia, mantenimiento y coste.
4. **Shadow mode:** ejecutar sin bloquear al menos sobre corpus dorado y una release previa.
5. **Decisión:** ADR con datos, amenazas, compatibilidad y rollback.
6. **Pin:** fijar versión/digest y generar inventario/SBOM.
7. **Graduación:** hacer blocking solo al cumplir umbral y tasa de falsos positivos aceptada.
8. **Revisión:** observar deriva; retirar o actualizar con el mismo proceso.

Una herramienta SaaS debe además documentar datos enviados, región/retención, disponibilidad, exportabilidad y modo degradado. Una herramienta basada en modelo debe fijar proveedor/modelo/configuración cuando sea posible y medir variabilidad.

## Cambios al SQ Plan

El plan usa SemVer. Correcciones editoriales incrementan patch; controles/alcance compatibles, minor; eliminación o redefinición de obligaciones, major. Ningún cambio de plan se aplica retroactivamente a evidencia histórica; el `GateRecord` conserva la versión evaluada.

