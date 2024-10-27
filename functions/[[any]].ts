import Html from "../lib/index.html"

function insertMetadata(data: Record<string, string>) {
	let meta = ''

	for (const key in data) {
		meta += `<meta name="${key}" content="${data[key]}">`
	}

	return Html.replace('<!-- META -->', meta)
}

export const onRequest: PagesFunction = async(context) => {
	const url = new URL(context.request.url)

	const meta: Record<string, string> = {
		'description': 'MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease.',
		'og:description': 'MCJars is a service that provides Minecraft server owners with the ability to download server jars and other files with ease.',
		'og:title': 'MCJars',
		'og:image': 'https://s3.mcjars.app/icons/vanilla.png',
		'og:url': context.request.url
	}

	if (url.pathname === '/') {
		return new Response(insertMetadata(meta), {
			headers: {
				'Content-Type': 'text/html'
			}
		})
	}

	const segments = url.pathname.split('/').filter(Boolean)

	if (segments[0] === 'assets') {
		return context.env.ASSETS.fetch(url)
	}

	const type = segments[0]
	const page = segments[1]

	if (type === 'lookup') {
		meta['og:title'] = 'MCJars | Reverse Lookup'
		meta['description'] = 'Lookup Minecraft server jars and configs by their hash.'
		meta['og:description'] = 'Lookup Minecraft server jars and configs by their hash.'
	} else {
		const { types } = await fetch('https://versions.mcjars.app/api/v1/types').then((res) => res.json() as any as {
			types: Record<string, {
				name: string
				builds: number
				versions: {
					minecraft: number
					project: number
				}
			}>
		})

		if (type === 'sitemap.xml') {
			let sitemap = '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'

			sitemap += `<url><loc>https://mcjars.app</loc></url>`
			sitemap += `<url><loc>https://mcjars.app/lookup</loc></url>`

			const mod = new Date().toISOString().split('T')[0]

			for (const type in types) {
				sitemap += `<url><loc>https://mcjars.app/${type}/versions</loc><lastmod>${mod}</lastmod></url>`
				sitemap += `<url><loc>https://mcjars.app/${type}/statistics</loc><lastmod>${mod}</lastmod></url>`
			}

			sitemap += '</urlset>'

			return new Response(sitemap, {
				headers: {
					'Content-Type': 'text/xml'
				}
			})
		}

		meta['og:image'] = `https://s3.mcjars.app/icons/${type.toLowerCase()}.png`

		const data = types[type.toUpperCase()]

		if (data) switch (page) {
			case "versions":
				meta['og:title'] = `MCJars | ${data.name} Versions`
				meta['description'] = `Download the latest ${data.name} server builds with ease. Browse ${data.builds} builds for ${data.versions.minecraft || data.versions.project} different versions on our website.`
				meta['og:description'] = `Download the latest ${data.name} server builds with ease. Browse ${data.builds} builds for ${data.versions.minecraft || data.versions.project} different versions on our website.`

				break
			case "statistics":
				meta['og:title'] = `MCJars | ${data.name} Statistics`
				meta['description'] = `View the latest statistics for ${data.name}.`
				meta['og:description'] = `View the latest statistics for ${data.name}.`

				break
		}
	}

	return new Response(insertMetadata(meta), {
		headers: {
			'Content-Type': 'text/html'
		}
	})
}