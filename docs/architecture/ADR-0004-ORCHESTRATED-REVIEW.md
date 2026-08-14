# ADR-0004: revisión independiente mediante orquestación de agentes

- Estado: aceptado
- Fecha: 2026-08-14
- Decisor: Product Owner
- Controles: `CTRL-GOV-002`, `CTRL-REPO-001`, `CTRL-EVID-001`
- Riesgos: `RISK-004`, `RISK-012`

## Contexto

Xunlie se opera inicialmente desde una sola identidad de GitHub, pero sus gates necesitan separar
autoría, verificación y aprobación técnica. Equiparar independencia con otra cuenta remota no
garantiza una revisión real; equiparar un check automático con un revisor tampoco registra alcance,
razonamiento adversarial ni findings.

La capacidad disponible es la orquestación de instancias de agente distintas. Cada instancia posee
una tarea acotada, puede reproducir controles de forma autónoma y entrega un veredicto al
orquestador. Esa separación debe ser explícita, revisable y fail-closed.

## Decisión

Se adopta `xunlie.review-orchestration/v1` como protocolo de independencia técnica:

1. el orquestador congela un candidate identificado por SHA;
2. asigna revisores que no participaron en la autoría y les prohíbe modificar el candidato;
3. registra para cada instancia identidad, tarea, alcance, candidate, comandos, findings, hora y
   veredicto;
4. los cambios críticos de dominio, protocolo, seguridad, CI o release requieren al menos dos
   revisores independientes; los demás, al menos uno;
5. un resultado positivo requiere unanimidad y cero findings críticos/altos abiertos;
6. un cambio posterior invalida los veredictos y obliga a revisar el nuevo candidate;
7. el orquestador agrega los resultados, pero no aporta un voto revisor.

El GateRecord conserva el quorum y enlaza un informe dentro del repositorio. La PR conserva la
atestación externa sobre el SHA final y GitHub aplica los checks y la protección de rama. La
titularidad, la intención de producto y la aceptación de riesgo jurídico o de negocio siguen
correspondiendo a `somefirenoodles`.

## Consecuencias

- La revisión es una capacidad del sistema, no una afirmación basada en confianza en el autor.
- La misma familia de modelo puede participar en varias instancias, pero no se interpreta como
  diversidad organizacional; se declara como independencia operacional y de tarea.
- Un revisor que edita el candidate pierde elegibilidad para aprobarlo.
- La evidencia permite reconstruir quién revisó qué SHA, con qué pruebas y resultado.
- Una segunda identidad humana futura puede añadirse como control adicional sin sustituir el
  protocolo ni reescribir evidencia histórica.

## Alternativas descartadas

- Autoaprobar con la única cuenta GitHub: no separa autoría y revisión.
- Considerar CI verde como aprobación: prueba controles, pero no realiza revisión adversarial.
- Bloquear toda entrega hasta incorporar otra persona: elimina una capacidad de orquestación ya
  disponible y no mejora por sí sola la calidad de la revisión.
- Permitir mayoría simple: puede ocultar un finding crítico; la política es no compensatoria.
