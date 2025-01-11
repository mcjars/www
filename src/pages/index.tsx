import apiGetTypeLookups from "@/api/lookups/types"
import apiGetVersionLookups from "@/api/lookups/versions"
import apiGetStats from "@/api/stats"
import apiGetVersionRequestsAllTime from "@/api/requests/version/all-time"
import apiGetVersionRequestsMonth from "@/api/requests/version/month"
import { Card } from "@/components/ui/card"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { mergeLessThanPercent } from "@/lib/utils"
import bytes from "bytes"
import { ArchiveIcon, ArchiveRestoreIcon, BrainIcon, DatabaseIcon, FileSearchIcon, GlobeIcon, LoaderCircle } from "lucide-react"
import { useMemo } from "react"
import { Bar, BarChart, CartesianGrid, Cell, Pie, PieChart, XAxis, YAxis } from "recharts"
import useSWR from "swr"
import { NumberParam, StringParam, useQueryParam } from "use-query-params"
import apiGetVersions from "@/api/versions"
import apiGetTypes from "@/api/types"
import { Alert, AlertDescription } from "@/components/ui/alert"

export default function PageIndex() {
	const { data: types } = useSWR(
		['types'],
		() => apiGetTypes(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const { data: stats } = useSWR(
    ['stats'],
    () => apiGetStats(),
    { revalidateOnFocus: false, revalidateIfStale: false }
  )

	const { data: versions } = useSWR(
		['versions', 'VANILLA'],
		() => apiGetVersions('VANILLA'),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const [ typeLookupsType, setTypeLookupsType ] = useQueryParam('typeLookupsType', StringParam)
	const { data: typeLookupsRaw } = useSWR(
		['typeLookups'],
		() => apiGetTypeLookups(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const typeLookups = useMemo(() => mergeLessThanPercent(
		Object.entries(typeLookupsRaw ?? {}).map(([ label, data ]) => ({ label, total: data[typeLookupsType === 'uniqueIps' ? 'uniqueIps' : 'total'] }))),
		[ typeLookupsRaw, typeLookupsType ]
	)

	const [ versionLookupsType, setVersionLookupsType ] = useQueryParam('versionLookupsType', StringParam)
	const { data: versionLookupsRaw } = useSWR(
		['versionLookups'],
		() => apiGetVersionLookups(),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const versionLookups = useMemo(() => mergeLessThanPercent(
		Object.entries(versionLookupsRaw ?? {}).map(([ label, data ]) => ({ label, total: data[versionLookupsType === 'uniqueIps' ? 'uniqueIps' : 'total'] }))).sort((a, b) => b.total - a.total),
		[ versionLookupsRaw, versionLookupsType ]
	)

	const [ requestVersionStatsMonthVersion, setRequestVersionStatsMonthVersion ] = useQueryParam('requestVersionStatsVersion', StringParam)
	const [ requestVersionStatsMonthType, setRequestVersionStatsMonthType ] = useQueryParam('requestVersionStatsMonthType', StringParam)
	const [ requestVersionStatsMonthDisplay, setRequestVersionStatsMonthDisplay ] = useQueryParam('requestVersionStatsMonthDisplay', StringParam)
	const [ requestVersionStatsMonthYear, setRequestVersionStatsMonthYear ] = useQueryParam('requestVersionStatsMonthYear', NumberParam)
	const [ requestVersionStatsMonthMonth, setRequestVersionStatsMonthMonth ] = useQueryParam('requestVersionStatsMonthMonth', NumberParam)
	const { data: requestVersionStatsMonthRaw } = useSWR(
		['requestVersionStatsMonth', requestVersionStatsMonthVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? '', requestVersionStatsMonthYear, requestVersionStatsMonthMonth],
		() => apiGetVersionRequestsMonth(requestVersionStatsMonthVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? '', requestVersionStatsMonthYear ?? new Date().getFullYear(), requestVersionStatsMonthMonth ?? new Date().getMonth() + 1),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const requestVersionStatsMonth = useMemo(() => Object.entries(requestVersionStatsMonthRaw?.requests ?? {}).map(([ label, days ]) => ({
		label,
		days,
		...days.reduce((acc, { day, total, uniqueIps }) => ({
			day,
			total: acc.total + total,
			uniqueIps: acc.uniqueIps + uniqueIps
		}), { total: 0, uniqueIps: 0 })
	})), [ requestVersionStatsMonthRaw ])

	const requestVersionStatsMonthMergedPercents = useMemo(() => mergeLessThanPercent(
		Object.entries(requestVersionStatsMonthRaw?.requests ?? {}).map(([ label, days ]) => ({
			label,
			total: days.reduce((acc, data) => acc + data[requestVersionStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'], 0),
			uniqueIps: days.reduce((acc, { uniqueIps }) => acc + uniqueIps, 0)
		}))
			.sort((a, b) => b.total - a.total)
	), [ requestVersionStatsMonthRaw, requestVersionStatsMonthType ])

	const [ requestVersionStatsAllTimeVersion, setRequestVersionStatsAllTimeVersion ] = useQueryParam('requestVersionStatsAllTimeVersion', StringParam)
	const [ requestVersionStatsAllTimeType, setRequestVersionStatsAllTimeType ] = useQueryParam('requestVersionStatsAllTimeType', StringParam)
	const { data: requestVersionStatsAllTimeRaw } = useSWR(
		['requestVersionStatsAllTime', requestVersionStatsAllTimeVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''],
		() => apiGetVersionRequestsAllTime(requestVersionStatsAllTimeVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''),
		{ revalidateOnFocus: false, revalidateIfStale: false }
	)

	const requestVersionStatsAllTime = useMemo(() => mergeLessThanPercent(
		Object.entries(requestVersionStatsAllTimeRaw?.requests ?? {}).map(([ label, data ]) => ({ label, total: data[requestVersionStatsAllTimeType === 'uniqueIps' ? 'uniqueIps' : 'total'] })).sort((a, b) => b.total - a.total)),
		[ requestVersionStatsAllTimeRaw, requestVersionStatsAllTimeType ]
	)

	return (
		<div>
			<Alert className={'mb-2'}>
				<AlertDescription>
					Welcome to <span className={'font-semibold'}>MCJars</span>! Use the sidebar on the left
					to navigate. This site will never have ads, and is open source
					on <a href={'https://github.com/mcjars/www'} className={'text-blue-500 font-semibold underline'} target={'_blank'} rel={'noopener noreferrer'}>GitHub</a>.
				</AlertDescription>
			</Alert>

			<div className={'grid gap-2 md:grid-cols-[repeat(auto-fit,minmax(30rem,1fr))] w-full'}>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<BrainIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.builds || <Skeleton className={'w-20 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>Total Builds</p>
					</div>
				</Card>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<FileSearchIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.hashes || <Skeleton className={'w-20 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>Total Hashes</p>
					</div>
				</Card>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<GlobeIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.requests || <Skeleton className={'w-20 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>Total Requests</p>
					</div>
				</Card>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<ArchiveIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.total.jarSize ? bytes(stats.total.jarSize) : <Skeleton className={'w-20 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>Total Jar Size</p>
					</div>
				</Card>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<ArchiveRestoreIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.total.zipSize ? bytes(stats.total.zipSize) : <Skeleton className={'w-20 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>Total Zip Size</p>
					</div>
				</Card>
				<Card className={'p-4 min-w-40 flex flex-row items-center justify-between'}>
					<DatabaseIcon className={'w-8 h-8'} />

					<div className={'flex flex-col text-right items-end'}>
						<h1 className={'text-xl font-semibold'}>
							{stats?.size.database ? bytes(stats.size.database) : <Skeleton className={'w-36 h-7'} />}
						</h1>
						<p className={'text-sm text-muted-foreground'}>SQL Database Size</p>
					</div>
				</Card>
			</div>
			<div className={'my-2 grid xl:grid-cols-2 grid-cols-1 gap-2 w-full'}>
				<Card className={'p-4 h-[500px]'}>
					<div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
						<h1 className={'text-xl font-semibold ml-2'}>All Time Lookup Statistics for Types</h1>

						<div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
							<Select value={typeLookupsType ?? 'total'} onValueChange={(value) => setTypeLookupsType(value)}>
								<SelectTrigger className={'w-[10em]'}>
									<SelectValue placeholder={'Total'} />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={'total'}>Total</SelectItem>
									<SelectItem value={'uniqueIps'}>Unique IPs</SelectItem>
								</SelectContent>
							</Select>
						</div>
					</div>

					{!typeLookups?.length ? (
						<div className={'w-full h-full flex flex-row items-center justify-center'}>
							<LoaderCircle className={'animate-spin'} />
						</div>
					) : (
						<ChartContainer config={{}} className={'w-full h-full'}>
							<PieChart accessibilityLayer>
								<ChartTooltip content={<ChartTooltipContent />} />
								<Pie
									data={typeLookups}
									dataKey={'total'}
									nameKey={'label'}
									fillRule={'evenodd'}
									label={({ name }) => name}
								>
									{typeLookups.map(({ label }, i) => (
										<Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
									))}
								</Pie>
							</PieChart>
						</ChartContainer>
					)}
				</Card>

				<Card className={'p-4 h-[500px]'}>
					<div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
						<h1 className={'text-xl font-semibold ml-2'}>All Time Lookup Statistics for Versions</h1>

						<div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
							<Select value={versionLookupsType ?? 'total'} onValueChange={(value) => setVersionLookupsType(value)}>
								<SelectTrigger className={'w-[10em]'}>
									<SelectValue placeholder={'Total'} />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={'total'}>Total</SelectItem>
									<SelectItem value={'uniqueIps'}>Unique IPs</SelectItem>
								</SelectContent>
							</Select>
						</div>
					</div>

					{!versionLookups?.length ? (
						<div className={'w-full h-full flex flex-row items-center justify-center'}>
							<LoaderCircle className={'animate-spin'} />
						</div>
					) : (
						<ChartContainer config={{}} className={'w-full h-full'}>
							<PieChart accessibilityLayer>
								<ChartTooltip content={<ChartTooltipContent />} />
								<Pie
									data={versionLookups}
									dataKey={'total'}
									nameKey={'label'}
									fillRule={'evenodd'}
									label={({ name }) => name}
								>
									{versionLookups.map(({ label }, i) => (
										<Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
									))}
								</Pie>
							</PieChart>
						</ChartContainer>
					)}
				</Card>

				<Card className={'p-4 h-[500px]'}>
					<div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
						<h1 className={'text-xl font-semibold ml-2'}>Monthly Request Statistics for {requestVersionStatsMonthVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''}</h1>

						<div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
							<Select value={requestVersionStatsMonthVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''} onValueChange={(value) => setRequestVersionStatsMonthVersion(value)}>
								<SelectTrigger className={'w-[8em] mb-1'}>
									<SelectValue placeholder={'Version'} />
								</SelectTrigger>
								<SelectContent>
									{versions?.map(({ latest }) => (
										<SelectItem key={latest.versionId} value={latest.versionId!}>{latest.versionId}</SelectItem>
									))}
								</SelectContent>
							</Select>

							<Select value={requestVersionStatsMonthType ?? 'total'} onValueChange={(value) => setRequestVersionStatsMonthType(value)}>
								<SelectTrigger className={'w-[8em] ml-1'}>
									<SelectValue placeholder={'Total'} />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={'total'}>Total</SelectItem>
									<SelectItem value={'uniqueIps'}>Unique IPs</SelectItem>
								</SelectContent>
							</Select>
						</div>
					</div>

					<div className={'absolute left-0 bottom-0 flex flex-row items-center justify-between p-2 pb-1 z-10'}>
						<Select value={String(requestVersionStatsMonthYear ?? new Date().getFullYear())} onValueChange={(value) => setRequestVersionStatsMonthYear(Number(value))}>
							<SelectTrigger className={'w-[6em] mr-1 mb-1'}>
								<SelectValue placeholder={'Year'} />
							</SelectTrigger>
							<SelectContent>
								{Array.from({ length: new Date().getFullYear() - 2023 }, (_, i) => 2024 + i).map((year) => (
									<SelectItem key={year} value={String(year)}>{year}</SelectItem>
								))}
							</SelectContent>
						</Select>

						<Select value={String(requestVersionStatsMonthMonth ?? new Date().getMonth() + 1)} onValueChange={(value) => setRequestVersionStatsMonthMonth(Number(value))}>
							<SelectTrigger className={'w-[10em] mr-1 mb-1'}>
								<SelectValue placeholder={'Month'} />
							</SelectTrigger>
							<SelectContent>
								{Array.from({ length: 12 }, (_, i) => i + 1).map((month) => (
									<SelectItem key={month} value={String(month)}>{Intl.DateTimeFormat('en-US', { month: 'long' }).format(new Date(2023, month - 1))}</SelectItem>
								))}
							</SelectContent>
						</Select>

						<Select value={requestVersionStatsMonthDisplay ?? 'pie'} onValueChange={(value) => setRequestVersionStatsMonthDisplay(value)}>
							<SelectTrigger className={'w-[8em] mr-1 mb-1'}>
								<SelectValue placeholder={'Pie'} />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={'pie'}>Pie</SelectItem>
								<SelectItem value={'history'}>History</SelectItem>
							</SelectContent>
						</Select>
					</div>

					{!requestVersionStatsMonth?.length || requestVersionStatsMonth.reduce((acc, { total }) => acc + total, 0) === 0 ? (
						requestVersionStatsMonthRaw ? (
							<div className={'w-full h-full flex flex-row items-center justify-center'}>
								<p className={'text-muted-foreground'}>No data available for this month.</p>
							</div>
						) : (
							<div className={'w-full h-full flex flex-row items-center justify-center'}>
								<LoaderCircle className={'animate-spin'} />
							</div>
						)
					) : (
						<ChartContainer config={{}} className={'w-full md:h-[calc(100%-5rem)] h-[calc(100%-13rem)] absolute bottom-8 -left-4'}>
							{!requestVersionStatsMonthDisplay || requestVersionStatsMonthDisplay === 'pie' ? (
								<PieChart accessibilityLayer>
									<ChartTooltip content={<ChartTooltipContent />} />
									<Pie
										data={requestVersionStatsMonthMergedPercents}
										dataKey={'total'}
										nameKey={'label'}
										fillRule={'evenodd'}
										label={({ name }) => name}
									>
										{requestVersionStatsMonth.map(({ label }, i) => (
											<Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
										))}
									</Pie>
								</PieChart>
							) : (
								<BarChart
									data={Object.values(requestVersionStatsMonth)[0].days.map(({ day }) => ({
										day,
										...Object.fromEntries(requestVersionStatsMonth.map(({ label, days }) => [
											label,
											days.find((d) => d.day === day)?.[requestVersionStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'] ?? 0
										]))
									}))}
									layout={'horizontal'}
								>
									<ChartTooltip content={<ChartTooltipContent />} />
									<CartesianGrid vertical={false} />
									<XAxis dataKey={'day'} type={'category'} />
									<YAxis type={'number'} />
									{Object.values(types ?? {}).flat().map((type) => (
										<Bar
											key={type.identifier}
											stackId={'types'}
											barSize={32}
											dataKey={type.identifier}
											fill={type.color}
											radius={2}
										/>
									))}
								</BarChart>
							)}
						</ChartContainer>
					)}
				</Card>

				<Card className={'p-4 h-[500px]'}>
					<div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
						<h1 className={'text-xl font-semibold ml-2'}>All Time Request Statistics for {requestVersionStatsAllTimeVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''}</h1>

						<div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
							<Select value={requestVersionStatsAllTimeVersion ?? versions?.find((v) => v.type === 'RELEASE')?.latest.versionId ?? ''} onValueChange={(value) => setRequestVersionStatsAllTimeVersion(value)}>
								<SelectTrigger className={'w-[8em] mb-1'}>
									<SelectValue placeholder={'Version'} />
								</SelectTrigger>
								<SelectContent>
									{versions?.map(({ latest }) => (
										<SelectItem key={latest.versionId} value={latest.versionId!}>{latest.versionId}</SelectItem>
									))}
								</SelectContent>
							</Select>

							<Select value={requestVersionStatsAllTimeType ?? 'total'} onValueChange={(value) => setRequestVersionStatsAllTimeType(value)}>
								<SelectTrigger className={'w-[8em] ml-1'}>
									<SelectValue placeholder={'Total'} />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={'total'}>Total</SelectItem>
									<SelectItem value={'uniqueIps'}>Unique IPs</SelectItem>
								</SelectContent>
							</Select>
						</div>
					</div>

					{!requestVersionStatsAllTime?.length ? (
						<div className={'w-full h-full flex flex-row items-center justify-center'}>
							<LoaderCircle className={'animate-spin'} />
						</div>
					) : (
						<ChartContainer config={{}} className={'w-full md:h-[calc(100%-5rem)] h-[calc(100%-10rem)] absolute bottom-8 -left-4'}>
							<PieChart accessibilityLayer>
								<ChartTooltip content={<ChartTooltipContent />} />
								<Pie
									data={requestVersionStatsAllTime}
									dataKey={'total'}
									nameKey={'label'}
									fillRule={'evenodd'}
									label={({ name }) => name}
								>
									{requestVersionStatsAllTime.map(({ label }, i) => (
										<Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
									))}
								</Pie>
							</PieChart>
						</ChartContainer>
					)}
				</Card>
			</div>
		</div>
	)
}