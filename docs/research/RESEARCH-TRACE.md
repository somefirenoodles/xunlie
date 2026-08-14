# Trazabilidad de investigación a diseño

La hipótesis de Xunlie procede de investigación reciente; los papers informan mecanismos y riesgos, no son requisitos normativos ni prueba de demanda universal.

| Fuente | Señal adoptada | Decisión de producto | Control contra sobreinterpretación |
|---|---|---|---|
| [SpecPath (arXiv:2608.09799)](https://arxiv.org/abs/2608.09799) | historias contractualmente equivalentes pueden producir resultados distintos | mantener contrato, base, agente/verificador y presupuesto fijos; variar historia | piloto propio, comparación pareada y distribución, no solo promedio |
| [Harness-IF (arXiv:2608.11727)](https://arxiv.org/abs/2608.11727) | superficie y precedencia de instrucciones son variables medibles | reglas atómicas, origen y precedencia explícitos | resolvedor y corpus de conflictos independiente del modelo |
| [DevIntent (arXiv:2608.07614)](https://arxiv.org/abs/2608.07614) | tests visibles pueden omitir restricciones de intención | ContractIR conserva obligaciones y verificadores ocultos/independientes | cobertura por obligación; un test aprobado no equivale a contrato completo |
| [Security Tests as Executable Specifications (arXiv:2608.09740)](https://arxiv.org/abs/2608.09740) | el oráculo visible tiene límites y efectos no uniformes | declarar alcance del oráculo y estado inconclusive | medir cobertura y separar visible/privado |
| [RETRACE (arXiv:2608.08950)](https://arxiv.org/abs/2608.08950) | verificación independiente del rationale puede añadir señal | verifier plugin de reconstrucción, fuera del agente | nunca sustituye tests duros; se evalúa tasa de error propia |

## Criterios de falsación del producto

Xunlie debe reconsiderarse si en pilotos representativos ocurre alguno:

- la divergencia desaparece al controlar ruido y oráculos;
- no puede certificarse equivalencia con precisión útil;
- el coste de variantes supera de forma persistente el valor de los escapes detectados;
- usuarios no pueden convertir requisitos reales en contratos verificables;
- productos existentes cubren la misma función con menor complejidad operacional.

## Vigilancia

El radar trimestral consulta cs.SE/cs.AI, fuentes primarias de estándares y repositorios oficiales. Se registra fecha, query, inclusión/exclusión y efecto sobre requisitos/riesgos. Una actualización de literatura no altera la baseline sin issue, evidencia y ADR cuando corresponda.

