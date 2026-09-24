---
title: "Errors and debugging"
description: "Handle QuiverAI API errors, correlate request IDs, inspect Logs, and escalate persistent failures."
icon: "circle-alert"
---

QuiverAI API failures use standard HTTP statuses and a machine-readable JSON envelope:

```json
{
  "status": 429,
  "code": "rate_limit_exceeded",
  "message": "Rate limit exceeded",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Branch on `status` and `code`, not on the human-readable `message`.

A request rejected for one particular parameter also carries `param`, that parameter as a path
from the body root:

```json
{
  "status": 400,
  "code": "invalid_request",
  "message": "Invalid value for 'store'",
  "param": "store",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

`param` names one parameter, the first the request failed on; correcting it can reveal another.
It is a parameter name, never a value you submitted. Nested parameters read as a path
(`size.width`, `input[0].content`). The key is absent when the failure is not about one particular
parameter, or when the parameter cannot be named, so a missing `param` is not an error.

A failure raised once streaming has started reports the same fields on its `event: error` frame,
in the shape each endpoint streams. The native SVG streams under `/v1/svgs/` send this envelope
unchanged. `/v1/responses` sends the Open Responses error event instead: the fields sit under
`error`, `status` is `error.status_code`, `error.param` is `null` rather than absent, and
`error.request_id` and `error.retry_after` carry the request id and the retry wait. A frame the
API itself sends there has `error.type` set to `quiver_error`, and its `sequence_number`
continues the stream's numbering.

On `/v1/responses`, an `event: error` frame the API itself sends is not the last event. It is
followed by `response.failed`, which carries the last response snapshot the stream sent and the
output items completed before the failure, and then by `data: [DONE]`. Without a response snapshot
to report, `data: [DONE]` follows the `event: error` frame directly. Read `code`, `status_code` and
`request_id` from the `event: error` frame; the `response.failed` after it reports the failure only
as `server_error`, with the same message.

Two messages distinguish the two ways a parameter can be wrong:

| Message                                                               | Meaning                                                          | What to do                       |
| --------------------------------------------------------------------- | ---------------------------------------------------------------- | -------------------------------- |
| `Invalid value for '<param>'`, optionally `Supported values are: ...` | The endpoint accepts this parameter, but not the value you sent. | Send one of the values it lists. |
| `Unrecognized request argument supplied: <param>`                     | The endpoint does not accept this parameter at all.              | Remove it from the request body. |

The second is the common failure when porting a request from another Responses implementation:
the endpoint rejects arguments it does not declare rather than ignoring them. `param` is the key
your request actually contained, so it also catches a misspelling.

When the parameter accepts a fixed set of values, the first message names them in a second
sentence, and the list is the complete set the endpoint accepts:

```json
{
  "status": 400,
  "code": "invalid_request",
  "message": "Invalid value for 'tools[0]'. Supported values are: 'function' and 'custom'.",
  "param": "tools[0]",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

A tool type, a `tool_choice`, and an enumerated option such as `reasoning.effort` all report this
way. The sentence lists what the endpoint accepts and never repeats what you sent. It is absent
when the parameter is not restricted to a fixed set — a free-form string, a number out of range, or
a value of the wrong type — so read its absence as "this parameter has no list", not as a different
class of failure.

## Failures found after the body parses

Some requests are well-formed and are still refused for one parameter. These name it too:

| Failure                                                                                 | `param`                                                                              |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `404` `model_not_found` — unknown or retired model, or one outside your key's authority | `model`                                                                              |
| `400` `Model does not support this operation` or `Model does not support SVG animation` | `model`                                                                              |
| `400` `max_output_tokens exceeds the selected model limit`                              | `max_output_tokens`                                                                  |
| `400` an unreadable reference image, or too many of them, on `POST /v1/responses`       | the path to the offending image, such as `input[0].content[1].image_url`, or `input` |

These are the requests that carry the model in their body. `param` is always a path from the
request body root, so a request that names the model in its URL instead — `GET /v1/models/{model}`
— reports no `param`: the field it would name is already the path you called.

A `model_not_found` naming `model` does not distinguish a typo from a model your key may not
reach; check the model against `GET /v1/models`, which lists what this key can call.

## Status and code guide

| Status         | Common codes                                                                                                                              | What to do                                                                                                                                                                                                                                                   |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `400`          | `invalid_request`                                                                                                                         | Correct the parameter named by `param`, or the missing, malformed, or incompatible parameters. Do not retry the same body unchanged.                                                                                                                         |
| `401`          | `invalid_api_key`, `unauthorized`                                                                                                         | Check the Bearer header, secret, key status, environment, project, and key authority.                                                                                                                                                                        |
| `402`          | `insufficient_credits`, `funding_pending`, `funding_payment_method_required`, `funding_payment_action_required`, `funding_payment_failed` | Resolve funding in [API Platform Billing](https://platform.quiver.ai/billing), then retry when the balance or payment action is settled.                                                                                                                     |
| `403`          | `unauthorized`, `account_frozen`, `content_policy_violation`, `invalid_request`                                                           | For `unauthorized`, check the key's allowed operations and models, then resolve any account or content-policy restriction. For `invalid_request`, the credential is accepted and this request is refused, so another key will not help — change the request. |
| `404`          | `model_not_found` or `invalid_request`                                                                                                    | Refresh `GET /v1/models` for model availability, and verify the endpoint path and HTTP method.                                                                                                                                                               |
| `408` or `504` | `request_timeout`                                                                                                                         | Treat the outcome as potentially uncertain. Before retrying, assess duplicate-work risk; Responses has no documented idempotency key.                                                                                                                        |
| `413`          | `payload_too_large`                                                                                                                       | Reduce the upload or request payload before retrying.                                                                                                                                                                                                        |
| `415`          | `invalid_request`                                                                                                                         | Send the body with a `Content-Type` of `application/json`. `POST /v1/responses` refuses a body it cannot read as JSON, including one sent with no `Content-Type` at all.                                                                                     |
| `429`          | `rate_limit_exceeded`, `operation_rate_limit_exceeded`, `weekly_limit_exceeded`                                                           | Respect `Retry-After` when present, add backoff and jitter, and review [Limits](https://platform.quiver.ai/limits).                                                                                                                                          |
| `500`          | `server_error`                                                                                                                            | Before retrying, assess duplicate-work risk; Responses has no documented idempotency key. Escalate persistent failures.                                                                                                                                      |
| `502`          | `model_error`                                                                                                                             | Before retrying, assess duplicate-work risk; Responses has no documented idempotency key. Escalate persistent failures.                                                                                                                                      |
| `503`          | `model_unavailable`                                                                                                                       | Before retrying, assess duplicate-work risk; Responses has no documented idempotency key. Escalate persistent failures.                                                                                                                                      |

## Correlate a request

Every response includes `X-Request-ID`. On an error response, it matches the envelope's
`request_id`. Record this value with the endpoint, model, status, code, and UTC timestamp.

You can also send an optional `x-trace-id` from your application. The API echoes it in
`X-Trace-ID`, letting you join QuiverAI request records with caller telemetry.

When request metadata is available under your data policy, use
[Logs](https://platform.quiver.ai/logs) to inspect the request outcome. Logs can help correlate a
request with its project, API key, endpoint, model, status, charge state, and public error code;
they do not capture prompts, request or response bodies, uploads, or generated artifacts. The
public error code is `invalid_request` for every rejected request body, so start from `param` on
the response itself to find which parameter was wrong; Logs tell you which request, not which
field.

## Interpret a rate-limit response

A `429` means an independently enforced capacity rejected the request. Use the canonical
[response-header definitions](/api-reference/introduction#response-headers) to interpret the
reported scope, subject, dimension, remaining capacity, reset time, and optional `Retry-After`.

A key that does not authorize the requested operation or model can reject with `403 unauthorized`.
When the capability is what the key lacks, the message names it and the remedy. A key's
capabilities and models are fixed when it is created, so the remedy is a new key from
[API Keys](https://platform.quiver.ai/api-keys); changing capacity settings does not grant a key
additional authority.

Honor `Retry-After` when it is present. When no retry timing is supplied and the reported capacity
is zero, the relevant limit must change before the request can succeed.

## Unsupported Responses options

The Responses endpoint is stateless. `store: true` and a non-null `previous_response_id` return
`400 invalid_request` before model execution, regardless of organization policy. Omit `store` or
send `false`; omit `previous_response_id` or send `null`. Continue by replaying the full input
history, not by retrieving a stored response. Changing data policy will not enable these features.

Remove unsupported fields such as `background`, `include`, and `websocket` before retrying. See
[Current limitations](/developers/guides/migrate-to-responses#current-limitations). Caller-tool
storage is your application's responsibility and is distinct from QuiverAI's metadata and
inference-retention policies in [Data controls](/developers/platform/data-controls).

## Streaming failures

Handle each failure boundary separately:

- **HTTP setup failure:** A non-`200` response before SSE begins uses the JSON error envelope.
- **Terminal SSE failure:** For `/v1/responses`, handle `response.failed`, `response.incomplete`,
  and `event: error`. Collect complete `function_call` or `custom_tool_call` items before handling
  them; partial arguments are not executable calls. A completed call can be followed by more
  refinement turns, so do not publish staged content until the tool loop finishes successfully.
- **Early EOF:** If the connection closes before a recognized terminal event, treat the outcome as
  uncertain and retain the request id for investigation.
- **Client cancellation or timeout:** Stopping the client does not prove server-side work stopped.
  Assess duplicate-work risk before submitting another Responses request because the endpoint has
  no documented idempotency key.
- Native SVG streams emit preview events before the final `content` event. Do not persist a preview.
- `data: [DONE]` ends the transport. Evaluate the events that preceded it to determine the request
  outcome. On `/v1/responses`, a stream that sent `event: error` failed, whatever follows it; take
  the failure's `code` and `request_id` from that frame.

## Test error handling

Test keys use the production base URL and published response shapes without model calls or charges.
Use the sandbox's deterministic outcomes and supported failure markers to exercise client handling.
The sandbox serves `/v1/responses` as well as the native endpoints; see
[Sandbox and test keys](/developers/guides/sandbox-and-test-keys).

## Escalate a persistent failure

Provide support with:

- the `X-Request-ID` or error `request_id`;
- UTC timestamp, endpoint, model, HTTP status, and machine-readable code;
- a minimal reproduction and sanitized request shape;
- whether the request was streaming and which terminal event arrived.

Never send an API key or other secret. Include prompts, source assets, or generated output only when
they are necessary to reproduce the issue and your data-sharing policy permits it.
