---
title: "Text to SVG"
description: "Generate SVG graphics from text prompts with optional reference images."
icon: "message"
sidebar:
  hidden: true
---

Generate one or more SVGs from a text prompt and optional reference images via `POST /v1/svgs/generations`.

Examples below use `arrow-2`. To discover model IDs available to your organization, call
`GET /v1/models`. Arrow 2 is the general-purpose release model; Arrow 2 Telos is the higher-fidelity
variant for detail-sensitive work.

Set `stream: true` to receive `reasoning`, `draft`, and `content` Server-Sent Events while the SVG
is being produced.

## Examples

![Elegant calligraphic script](/images/example-calligraphy.svg)

Calligraphy

![Japanese crane illustration](/images/example-crane.svg)

Illustration

![Heraldic lion crest](/images/example-heraldic-lion.svg)

Logo

### Calligraphy

**Prompt:** _"Elegant calligraphic script in a flowing hand-lettered style, single continuous stroke"_

**Node.js SDK**

```typescript
import { QuiverAI } from "@quiverai/sdk";

const client = new QuiverAI({
  bearerAuth: process.env["QUIVERAI_API_KEY"],
});

const result = await client.createSVGs.generateSVG({
  generateSVGRequest: {
    model: "arrow-2",
    prompt: "Elegant calligraphic script in a flowing hand-lettered style, single continuous stroke",
  },
});
```

**cURL**

```bash
curl --request POST \
  --url https://api.quiver.ai/v1/svgs/generations \
  --header 'Authorization: Bearer <QUIVERAI_API_KEY>' \
  --header 'Content-Type: application/json' \
  --data '{
    "model": "arrow-2",
    "prompt": "Elegant calligraphic script in a flowing hand-lettered style, single continuous stroke"
  }'
```

### Illustration

**Prompt:** _"Japanese crane in traditional woodblock illustration style with warm earth tones"_

```typescript
const result = await client.createSVGs.generateSVG({
  generateSVGRequest: {
    model: "arrow-2",
    prompt: "Japanese crane in traditional woodblock illustration style with warm earth tones",
    instructions: "Use a warm muted palette with detailed feather work",
  },
});
```

### Logo

**Prompt:** _"Heraldic lion crest with ornate medieval style details and gold gradient accents"_

```typescript
const result = await client.createSVGs.generateSVG({
  generateSVGRequest: {
    model: "arrow-2",
    prompt: "Heraldic lion crest with ornate medieval style details and gold gradient accents",
  },
});
```

## Choosing a model

- Use `arrow-2` for most text-to-SVG generation. It balances quality and speed while keeping
  outputs clean and editable.
- Use `arrow-2-telos` when output quality is the priority. It is better suited to dense
  illustrations, logos with tight geometry, technical diagrams, and other compositions where
  precision matters more than speed.

## Writing prompts

For an example-led introduction to briefs, reference images, structured prompt text, and focused
revisions, read the App [Prompt guide](/app/prompt-guide).

Include these elements for the best results:

- **Subject:** What is in the image? Be specific. For example, "a logo for an eco-friendly coffee company".
- **Style:** What is the overall aesthetic? For example, "line art", "hand drawn", "duotone", or "flat monochrome icon".
- **Color palette:** Which colors should be used? For example, "background: #e9edc9 and logo in #fb8500".
- **Composition:** Include framing details like "centered icon" or "wide horizontal logo".
- **Text integration:** Clearly state what text should appear and how. For example, "The headline 'URBAN EXPLORER' in bold white sans-serif at the top".

You can also use the `instructions` parameter to provide separate style or formatting guidance without mixing it into the prompt.

## Reference images

Reference images can be provided as `{ url: "..." }`, `{ base64: "..." }`, or URL string shorthand.
Direct base64 image payloads can be PNG, JPEG, WebP, GIF, or SVG. Decoded image inputs must be no
larger than 12,582,912 bytes, 4096 x 4096 pixels, or 16,777,216 total pixels.

Image URLs must use HTTP or HTTPS, resolve to a public network target, and return an image content
type. The API follows up to 3 redirects and applies the same decoded image limits after fetching.

## Parameters

| Parameter           | Type    | Default | Description                                                                                                                                                   |
| ------------------- | ------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`             | string  | --      | **Required.** Model identifier (for example, `arrow-2` or `arrow-2-telos`).                                                                                   |
| `prompt`            | string  | --      | **Required.** Text description of the desired SVG.                                                                                                            |
| `instructions`      | string  | --      | Additional style or formatting guidance, separate from the prompt.                                                                                            |
| `references`        | array   | --      | Reference images, provided by URL or base64, to guide generation. Arrow 2 and Arrow 2 Telos support up to 14 references.                                      |
| `n`                 | integer | `1`     | Number of outputs to generate (1 to 16).                                                                                                                      |
| `stream`            | boolean | `false` | When `true`, returns a Server-Sent Events stream with progressive rendering phases (`reasoning`, `draft`, `content`).                                         |
| `temperature`       | number  | `1`     | Controls randomness (0 to 2). Lower values produce more deterministic output; higher values increase variety.                                                 |
| `top_p`             | number  | `1`     | Nucleus sampling (0 to 1). Limits token selection to the smallest set whose cumulative probability exceeds this value. Lower values make output more focused. |
| `presence_penalty`  | number  | `0`     | Penalizes tokens already present in prior output (-2 to 2). Positive values encourage the model to explore new patterns.                                      |
| `max_output_tokens` | integer | --      | Upper bound for output token count (1 to 65536).                                                                                                              |
