import { NextRequest, NextResponse } from "next/server";

const ORCHESTRATOR_URL =
  process.env.ORCHESTRATOR_URL ?? "http://127.0.0.1:8080";

async function proxyRequest(
  req: NextRequest,
  params: { path: string[] },
): Promise<NextResponse> {
  const targetPath = params.path.join("/");
  const targetUrl = new URL(`${ORCHESTRATOR_URL}/${targetPath}`);
  targetUrl.search = req.nextUrl.search;

  const contentType = req.headers.get("content-type");
  const body =
    req.method === "GET" || req.method === "HEAD"
      ? undefined
      : contentType?.includes("application/json")
        ? JSON.stringify(await req.json())
        : await req.text();

  let upstream: Response;

  try {
    upstream = await fetch(targetUrl, {
      method: req.method,
      headers: {
        ...(contentType ? { "content-type": contentType } : {}),
      },
      body,
      cache: "no-store",
    });
  } catch {
    return NextResponse.json(
      {
        error: "Orchestrator unavailable",
        message: `Could not reach ${ORCHESTRATOR_URL}`,
      },
      { status: 503 },
    );
  }

  const rawText = await upstream.text();

  return new NextResponse(rawText, {
    status: upstream.status,
    headers: {
      "content-type": upstream.headers.get("content-type") ?? "application/json",
    },
  });
}

export async function GET(
  req: NextRequest,
  { params }: { params: Promise<{ path: string[] }> },
) {
  return proxyRequest(req, await params);
}

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ path: string[] }> },
) {
  return proxyRequest(req, await params);
}
