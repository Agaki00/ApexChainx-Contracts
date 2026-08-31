Problem
correlation_event_topics (src/event_correlation.rs) takes a correlation id and returns topics without it:

pub fn correlation_event_topics(
    event_name: Symbol, event_version: Symbol, context: Symbol, _correlation_id: CorrelationId,
) -> (Symbol, Symbol, Symbol) {
    (event_name, event_version, context)
}
The doc comment explains the id is "carried in the event payload instead" — but, as the companion issue documents, no payload carries it either.

Consequences:

The helper's contract is empty: its only behavior is to build the standard 3-topic tuple; the correlation id parameter is decorative, so any caller that passes a real id gets nothing back — the function cannot be used to propagate correlation across boundaries.
The doc and the code disagree about where the id goes: the topic function says "payload instead", the payload function (publish_settlement_intent_event) has no id — the id has no home.
The function name overstates its role: "correlation_event_topics" implies the topics encode the correlation; they do not.
Root cause
The helper was written to satisfy the "3-topic arity" convention while the correlation payload was never added; the parameter was kept for API shape.

Why this is architecturally hard
The fix depends entirely on the correlation-wiring decision (companion issue): if the id goes in the payload, this helper should be deleted (it adds nothing); if the id goes in a topic, the 3-topic layout documented in src/event_schema.rs changes and this helper becomes the place that encodes it.
The helper is public library API with tests asserting its current (no-op) behavior; removing or repurposing it must update those tests and the coordination harness.
This is a good candidate for the smallest possible resolution: delete the dead parameter and document that correlation is payload-only, or implement the payload.
Acceptance criteria
 The helper either encodes the correlation id or is removed; no public function accepts a parameter it ignores.
 The correlation id's home (topic or payload) is documented in one place.
 Tests reflect the chosen behavior.



 