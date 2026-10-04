import { CheckIcon, CopyIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from '~/components/ui/button.tsx';
import { Card } from '~/components/ui/card.tsx';
import { useToast } from '~/hooks/use-toast.ts';

const TOOLS = [
  ['list_server_types', 'Every server type MCJars indexes, with descriptions and build counts.'],
  ['list_versions', 'The versions of a server type, with their required Java version.'],
  ['list_builds', 'The builds of a version, with changelogs and download links.'],
  ['get_latest_build', 'The newest build of a version.'],
  ['lookup_build', 'Identify a server jar by its hash or uuid.'],
  ['get_build_configs', 'The default config files a build ships with.'],
];

function CodeBlock({ code }: { code: string }) {
  const { toast } = useToast();
  const [copied, setCopied] = useState(false);

  return (
    <div className={'relative mt-2'}>
      <pre className={'border rounded-md bg-sidebar p-3 pr-12 text-xs overflow-x-auto'}>
        <code>{code}</code>
      </pre>
      <Button
        size={'icon'}
        variant={'ghost'}
        className={'absolute top-1 right-1 h-8 w-8'}
        onClick={() => {
          navigator.clipboard.writeText(code).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
            toast({ title: 'Copied', description: 'Copied to your clipboard.' });
          });
        }}
      >
        {copied ? <CheckIcon size={16} /> : <CopyIcon size={16} />}
      </Button>
    </div>
  );
}

function ClientCard({ name, children }: { name: string; children: React.ReactNode }) {
  return (
    <Card className={'p-4 w-full'}>
      <h2 className={'text-lg font-semibold mb-1'}>{name}</h2>
      <div className={'text-sm text-muted-foreground flex flex-col gap-2'}>{children}</div>
    </Card>
  );
}

export default function PageAi() {
  const [url, setUrl] = useState('https://mcjars.app/mcp');

  useEffect(() => {
    setUrl(`${window.location.origin}/mcp`);
  }, []);

  const cursorConfig = btoa(JSON.stringify({ url }));
  const vscodeConfig = encodeURIComponent(JSON.stringify({ name: 'mcjars', type: 'http', url }));

  return (
    <div className={'w-full flex flex-col gap-2 pb-4'}>
      <Card className={'p-4 w-full'}>
        <h1 className={'text-2xl font-bold mb-2'}>AI Assistants</h1>
        <p className={'text-sm text-muted-foreground'}>
          MCJars runs a public{' '}
          <a href={'https://modelcontextprotocol.io'} target={'_blank'} rel={'noreferrer'} className={'underline'}>
            Model Context Protocol
          </a>{' '}
          server, so AI assistants can look up server software, versions, builds, download links and default configs
          for you. It needs no account or API key and is read-only.
        </p>
        <CodeBlock code={url} />

        <div className={'grid md:grid-cols-2 gap-x-4 gap-y-1 mt-4 text-sm'}>
          {TOOLS.map(([name, description]) => (
            <p key={name}>
              <code className={'border rounded-md px-1'}>{name}</code>{' '}
              <span className={'text-muted-foreground'}>{description}</span>
            </p>
          ))}
        </div>
      </Card>

      <ClientCard name={'Claude Code'}>
        <p>
          Run this in your terminal. Add <code>--scope user</code> to enable it in every project, or{' '}
          <code>--scope project</code> to share it through <code>.mcp.json</code>.
        </p>
        <CodeBlock code={`claude mcp add --transport http mcjars ${url}`} />
      </ClientCard>

      <ClientCard name={'Claude (claude.ai and Claude Desktop)'}>
        <p>
          Open <b>Customize</b> → <b>Connectors</b>, press <b>Add</b> → <b>Add custom connector</b>, enter{' '}
          <code>MCJars</code> as the name and the URL below, then choose <b>No sign in</b>. Connectors added on
          claude.ai sync to Claude Desktop. On Team and Enterprise plans an owner has to add it under organization
          settings first.
        </p>
        <CodeBlock code={url} />
      </ClientCard>

      <ClientCard name={'ChatGPT'}>
        <p>
          Turn on <b>Developer mode</b> under <b>Settings</b> → <b>Security and login</b>, then add a new app or
          connector and use the URL below as its connection, with no authentication. Availability depends on your plan
          and workspace policy.
        </p>
        <CodeBlock code={url} />
      </ClientCard>

      <ClientCard name={'Cursor'}>
        <p>
          <a
            href={`cursor://anysphere.cursor-deeplink/mcp/install?name=mcjars&config=${cursorConfig}`}
            className={'underline'}
          >
            Install in Cursor
          </a>
          , or add this to <code>~/.cursor/mcp.json</code> (all projects) or <code>.cursor/mcp.json</code> (one
          project):
        </p>
        <CodeBlock code={JSON.stringify({ mcpServers: { mcjars: { url } } }, null, 2)} />
      </ClientCard>

      <ClientCard name={'VS Code (GitHub Copilot)'}>
        <p>
          <a href={`vscode:mcp/install?${vscodeConfig}`} className={'underline'}>
            Install in VS Code
          </a>
          , or add this to <code>.vscode/mcp.json</code> or your user configuration (<b>MCP: Open User Configuration</b>
          ). Note the top-level key is <code>servers</code>.
        </p>
        <CodeBlock code={JSON.stringify({ servers: { mcjars: { type: 'http', url } } }, null, 2)} />
      </ClientCard>

      <ClientCard name={'Windsurf'}>
        <p>
          Add this to your <code>mcp_config.json</code>, which you can open from the MCP settings in Cascade:
        </p>
        <CodeBlock code={JSON.stringify({ mcpServers: { mcjars: { serverUrl: url } } }, null, 2)} />
      </ClientCard>

      <ClientCard name={'Codex CLI'}>
        <p>Run this in your terminal:</p>
        <CodeBlock code={`codex mcp add mcjars --url ${url}`} />
        <p>
          Or add this to <code>~/.codex/config.toml</code>:
        </p>
        <CodeBlock code={`[mcp_servers.mcjars]\nurl = "${url}"`} />
      </ClientCard>

      <ClientCard name={'Gemini CLI'}>
        <p>Run this in your terminal:</p>
        <CodeBlock code={`gemini mcp add --transport http mcjars ${url}`} />
        <p>
          Or add this to your <code>settings.json</code>. Use <code>httpUrl</code>, since <code>url</code> means the
          older SSE transport:
        </p>
        <CodeBlock code={JSON.stringify({ mcpServers: { mcjars: { httpUrl: url } } }, null, 2)} />
      </ClientCard>

      <ClientCard name={'Zed'}>
        <p>
          Add this to your Zed <code>settings.json</code>:
        </p>
        <CodeBlock code={JSON.stringify({ context_servers: { mcjars: { url } } }, null, 2)} />
      </ClientCard>

      <ClientCard name={'Other clients'}>
        <p>
          Any client that supports remote MCP servers over Streamable HTTP can connect to the URL above without
          authentication.
        </p>
      </ClientCard>

      <ClientCard name={'In your browser (WebMCP)'}>
        <p>
          This site also registers the same tools through{' '}
          <a
            href={'https://webmachinelearning.github.io/webmcp/'}
            target={'_blank'}
            rel={'noreferrer'}
            className={'underline'}
          >
            WebMCP
          </a>
          , so browser agents that support it can use them directly while you have MCJars open, without any setup.
          WebMCP is still experimental and only available in some browsers.
        </p>
      </ClientCard>
    </div>
  );
}
