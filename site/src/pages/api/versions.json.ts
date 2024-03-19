import type { APIRoute } from "astro"
import versions from "~root/versions/versions.json" with { type: "json" }

export const GET: APIRoute = () => new Response(JSON.stringify(versions, null, 2))
