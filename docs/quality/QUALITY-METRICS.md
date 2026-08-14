# Métricas, score y señales

## Regla de lectura

Las métricas observan el sistema; los gates deciden. El score no puede convertir un fallo crítico en aprobación.

## Score de completitud de etapa

Para una etapa con controles aplicables:

```text
evidence_completeness = evidencias válidas / evidencias requeridas
control_completion     = controles PASS / controles aplicables
traceability           = enlaces completos / enlaces requeridos

score = 100 × (0.40 × control_completion
             + 0.35 × evidence_completeness
             + 0.25 × traceability)
```

La etapa es `PASS` únicamente si:

- `score = 100`;
- todo control `MANDATORY` está en `PASS`;
- no hay waivers vencidos ni blockers;
- el aprobador es independiente.

Un control no aplicable requiere razón registrada y aprobada; no desaparece del denominador sin decisión.

## Cuadro de mando

| Dimensión | Indicador | Objetivo inicial | Acción |
|---|---|---:|---|
| Intención | requisitos con trazabilidad completa | 100% | bloquear |
| Arquitectura | fitness functions obligatorias | 100% PASS | bloquear |
| Funcional | aceptación por requisito | 100% PASS | bloquear |
| Código | línea/rama global | ≥90%/≥85% | bloquear nueva caída; investigar tendencia |
| Robustez | mutation score crítico/global | ≥95%/≥85% | bloquear crítico |
| Seguridad | críticos/altos confirmados | 0 | bloquear release |
| Supply chain | dependencias sin origen/licencia permitida | 0 | bloquear |
| Confiabilidad | flaky tests requeridos | 0 | bloquear release |
| Auditabilidad | bundles reproducibles | 100% de muestra | bloquear |
| Flujo | mediana PR lead time | observar baseline | mejorar sin relajar gates |
| Cambio | change failure rate | <10% tras baseline | CAPA por tendencia |
| Operación | MTTR Sev-1/2 | objetivo tras piloto | revisar capacidad |

## Métricas específicas del producto

- PDR por agente, adaptador, modelo, tipo de historia y versión de contrato;
- cobertura de oráculo por obligación;
- tasa de corridas `inconclusive` y causa;
- varianza entre repeticiones de la misma configuración;
- ratio de fallos de infraestructura separado de fallos semánticos;
- tiempo, tokens/coste y recursos por variante;
- tasa de certificados de equivalencia rechazados;
- reproducibilidad de replay por plataforma;
- falsos positivos/negativos confirmados del gate.

PDR nunca se publica sin tamaño de muestra, intervalo o distribución, cobertura de oráculo y versión de configuración.

## Anti-métricas

No se usan como objetivos aislados: número de commits, líneas producidas, velocidad del agente, cantidad de tests, issues cerrados o cobertura 100%. Incentivan volumen y pueden degradar corrección.

## Calidad de los datos

Cada métrica declara fórmula, unidad, fuente, ventana, dueño y limitaciones. Un cambio de fórmula crea nueva versión de métrica; las series no se mezclan silenciosamente.

