const MCP_ROUTE = '/mcp';
const PROTOCOL_VERSION = '2025-06-18';

interface ToolAnnotations {
  readOnlyHint?: boolean;
}

interface ModelContextTool {
  name: string;
  title?: string;
  description: string;
  inputSchema?: object;
  annotations?: ToolAnnotations;
  execute(input: object, options?: { signal?: AbortSignal }): Promise<unknown>;
}

interface ModelContext {
  registerTool(tool: ModelContextTool): Promise<undefined> | undefined;
}

interface McpTool {
  name: string;
  title?: string;
  description?: string;
  inputSchema?: object;
  annotations?: ToolAnnotations & { title?: string };
}

interface ContentBlock {
  type: string;
  text?: string;
}

interface CallToolResult {
  content?: ContentBlock[];
  isError?: boolean;
}

interface RpcResponse {
  id?: number | string | null;
  result?: unknown;
  error?: { code: number; message: string };
}

let nextId = 1;
let registered = false;

function modelContext(): ModelContext | null {
  const host = ('modelContext' in document ? document : 'modelContext' in navigator ? navigator : null) as {
    modelContext?: ModelContext;
  } | null;
  return host?.modelContext ?? null;
}

function sseData(body: string): string[] {
  const events: string[] = [];
  let data: string[] = [];

  for (const line of body.split(/\r?\n/)) {
    if (line === '') {
      if (data.length > 0) events.push(data.join('\n'));
      data = [];
      continue;
    }
    if (line.startsWith('data:')) data.push(line.slice(5).replace(/^ /, ''));
  }
  if (data.length > 0) events.push(data.join('\n'));

  return events;
}

async function rpc(method: string, params: object, signal?: AbortSignal): Promise<unknown> {
  const id = nextId++;
  const response = await fetch(MCP_ROUTE, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json, text/event-stream',
      'MCP-Protocol-Version': PROTOCOL_VERSION,
    },
    body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
    signal,
  });
  const body = await response.text();
  if (!response.ok) throw new Error(`${MCP_ROUTE} answered HTTP ${response.status} to ${method}: ${body}`);

  const messages = (response.headers.get('Content-Type') ?? '').includes('text/event-stream') ? sseData(body) : [body];

  for (const raw of messages) {
    let message: RpcResponse;
    try {
      message = JSON.parse(raw) as RpcResponse;
    } catch {
      continue;
    }
    if (message.id !== id) continue;
    if (message.error) throw new Error(message.error.message);
    return message.result;
  }

  throw new Error(`${MCP_ROUTE} sent no response to ${method}`);
}

function unwrap(result: CallToolResult): unknown {
  const content = result.content ?? [];
  const text = content
    .filter((block) => block.type === 'text')
    .map((block) => block.text ?? '')
    .join('\n\n');

  if (result.isError) return text || 'The tool call failed.';
  if (!content.every((block) => block.type === 'text')) return content;
  if (content.length === 1) {
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  return text;
}

function proxied(context: ModelContext, tool: McpTool): Promise<undefined> | undefined {
  return context.registerTool({
    name: tool.name,
    title: tool.title ?? tool.annotations?.title,
    description: tool.description ?? '',
    inputSchema: tool.inputSchema,
    annotations: tool.annotations && { readOnlyHint: tool.annotations.readOnlyHint ?? false },
    execute: async (input, options) =>
      unwrap((await rpc('tools/call', { name: tool.name, arguments: input }, options?.signal)) as CallToolResult),
  });
}

export async function registerWebMcpTools(): Promise<void> {
  if (registered) return;
  const context = modelContext();
  if (context === null) return;
  registered = true;

  try {
    const { tools } = (await rpc('tools/list', {})) as { tools: McpTool[] };
    await Promise.all(tools.map((tool) => proxied(context, tool)));
  } catch (error) {
    registered = false;
    console.warn('WebMCP tools were not registered:', error);
  }
}
