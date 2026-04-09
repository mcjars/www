import { CodeIcon, FileTextIcon, FileIcon, HashIcon } from 'lucide-react'

export default function FileFormatIcon({ format, className, size = 16 }: { format?: string; className?: string; size?: number }) {
	const f = (format ?? '').toLowerCase()

	if (f === 'json' || f === 'json5') {
		return <CodeIcon size={size} className={className} />
	}

	if (f === 'yaml' || f === 'yml') {
		return <FileTextIcon size={size} className={className} />
	}

	if (f === 'toml') {
		return <HashIcon size={size} className={className} />
	}

	return <FileIcon size={size} className={className} />
}