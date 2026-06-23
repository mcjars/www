import bytes from 'bytes';
import { ArchiveIcon, ArchiveRestoreIcon, LoaderCircle } from 'lucide-react';
import { useCallback, useMemo } from 'react';
import { useParams, useSearchParams } from 'react-router';
import { Bar, BarChart, CartesianGrid, Cell, Pie, PieChart, XAxis, YAxis } from 'recharts';
import { useQuery } from '@tanstack/react-query';
import apiGetTypeVersionLookupsAllTime from '~/api/lookups/version/all-time.ts';
import apiGetTypeVersionLookupsMonth from '~/api/lookups/version/month.ts';
import apiGetTypeRequestsAllTime from '~/api/requests/type/all-time.ts';
import apiGetTypeRequestsMonth from '~/api/requests/type/month.ts';
import apiGetTypeStats from '~/api/requests/type/stats.ts';
import { Card } from '~/components/ui/card.tsx';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '~/components/ui/chart.tsx';
import { SearchableSelect } from '~/components/searchable-select.tsx';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select.tsx';
import { Skeleton } from '~/components/ui/skeleton.tsx';
import { mergeLessThanPercent } from '~/lib/utils.ts';

export default function PageTypeRequestStatistics() {
  const { type = '' } = useParams<{ type: string }>();

  const [searchParams, setSearchParams] = useSearchParams();
  const setParam = useCallback(
    (name: string, value: string | number | null | undefined) => {
      setSearchParams(
        (prev) => {
          const params = new URLSearchParams(prev);
          if (value === null || value === undefined || value === '') params.delete(name);
          else params.set(name, String(value));
          return params;
        },
        { replace: true },
      );
    },
    [setSearchParams],
  );
  const getNumberParam = (name: string) => {
    const raw = searchParams.get(name);
    return raw === null || raw === '' || Number.isNaN(Number(raw)) ? undefined : Number(raw);
  };

  const { data: stats } = useQuery({ queryKey: ['stats', type], queryFn: () => apiGetTypeStats(type) });

  const { data: requestStatsAllTimeRaw } = useQuery({
    queryKey: ['requestStats', type],
    queryFn: () => apiGetTypeRequestsAllTime(type),
  });

  const requestStatsAllTimeType = searchParams.get('requestStatsAllTimeType') ?? undefined;
  const setRequestStatsAllTimeType = (value?: string | null) => setParam('requestStatsAllTimeType', value);
  const requestStatsAllTime = useMemo(
    () =>
      mergeLessThanPercent(
        Object.entries(requestStatsAllTimeRaw?.versions ?? {}).map(([label, data]) => ({
          label,
          total: data[requestStatsAllTimeType === 'uniqueIps' ? 'uniqueIps' : 'total'],
        })),
      ).sort((a, b) => b.total - a.total),
    [requestStatsAllTimeRaw, requestStatsAllTimeType],
  );

  const { data: lookupVersionStatsAllTimeRaw } = useQuery({
    queryKey: ['lookupVersionStats', type],
    queryFn: () => apiGetTypeVersionLookupsAllTime(type),
  });

  const lookupVersionStatsAllTimeType = searchParams.get('lookupVersionStatsAllTimeType') ?? undefined;
  const setLookupVersionStatsAllTimeType = (value?: string | null) =>
    setParam('lookupVersionStatsAllTimeType', value);
  const lookupVersionStatsAllTime = useMemo(
    () =>
      mergeLessThanPercent(
        Object.entries(lookupVersionStatsAllTimeRaw ?? {}).map(([label, data]) => ({
          label,
          total: data[lookupVersionStatsAllTimeType === 'uniqueIps' ? 'uniqueIps' : 'total'],
        })),
      ).sort((a, b) => b.total - a.total),
    [lookupVersionStatsAllTimeRaw, lookupVersionStatsAllTimeType],
  );

  const requestStatsMonthType = searchParams.get('requestStatsMonthType') ?? undefined;
  const setRequestStatsMonthType = (value?: string | null) => setParam('requestStatsMonthType', value);
  const requestStatsMonthDisplay = searchParams.get('requestStatsMonthDisplay') ?? undefined;
  const setRequestStatsMonthDisplay = (value?: string | null) => setParam('requestStatsMonthDisplay', value);
  const requestStatsMonthYear = getNumberParam('requestStatsMonthYear');
  const setRequestStatsMonthYear = (value?: number | null) => setParam('requestStatsMonthYear', value);
  const requestStatsMonthMonth = getNumberParam('requestStatsMonthMonth');
  const setRequestStatsMonthMonth = (value?: number | null) => setParam('requestStatsMonthMonth', value);
  const { data: requestStatsMonthRaw } = useQuery({
    queryKey: ['requestStatsMonth', type, requestStatsMonthYear, requestStatsMonthMonth],
    queryFn: () =>
      apiGetTypeRequestsMonth(
        type,
        requestStatsMonthYear ?? new Date().getFullYear(),
        requestStatsMonthMonth ?? new Date().getMonth() + 1,
      ),
  });

  const requestStatsMonth = useMemo(
    () =>
      requestStatsMonthRaw?.map(({ day, root, versions }) => ({
        day,
        root: root[requestStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'],
        versions: Object.entries(versions).reduce(
          (acc, [label, data]) => ({
            ...acc,
            [label]: data[requestStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'],
          }),
          {},
        ) as Record<string, number>,
      })),
    [requestStatsMonthRaw, requestStatsMonthType],
  );

  const requestStatsMonthMergedPercents = useMemo(
    () =>
      mergeLessThanPercent(
        Object.entries(
          requestStatsMonthRaw?.reduce(
            (acc, { versions }) => {
              Object.entries(versions).forEach(([version, data]) => {
                if (!acc[version]) {
                  acc[version] = { total: 0, uniqueIps: 0 };
                }

                acc[version].total += data.total;
                acc[version].uniqueIps += data.uniqueIps;
              });

              return acc;
            },
            {} as Record<string, { total: number; uniqueIps: number }>,
          ) ?? {},
        )
          .map(([label, data]) => ({
            label,
            total: data[requestStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'],
          }))
          .sort((a, b) => b.total - a.total),
      ),
    [requestStatsMonth],
  );

  const lookupVersionStatsMonthType = searchParams.get('lookupVersionStatsMonthType') ?? undefined;
  const setLookupVersionStatsMonthType = (value?: string | null) => setParam('lookupVersionStatsMonthType', value);
  const lookupVersionStatsMonthDisplay = searchParams.get('lookupVersionStatsMonthDisplay') ?? undefined;
  const setLookupVersionStatsMonthDisplay = (value?: string | null) =>
    setParam('lookupVersionStatsMonthDisplay', value);
  const lookupVersionStatsMonthYear = getNumberParam('lookupVersionStatsMonthYear');
  const setLookupVersionStatsMonthYear = (value?: number | null) => setParam('lookupVersionStatsMonthYear', value);
  const lookupVersionStatsMonthMonth = getNumberParam('lookupVersionStatsMonthMonth');
  const setLookupVersionStatsMonthMonth = (value?: number | null) => setParam('lookupVersionStatsMonthMonth', value);
  const { data: lookupVersionStatsMonthRaw } = useQuery({
    queryKey: ['lookupVersionStatsMonth', type, lookupVersionStatsMonthYear, lookupVersionStatsMonthMonth],
    queryFn: () =>
      apiGetTypeVersionLookupsMonth(
        type,
        lookupVersionStatsMonthYear ?? new Date().getFullYear(),
        lookupVersionStatsMonthMonth ?? new Date().getMonth() + 1,
      ),
  });

  const lookupVersionStatsMonth = useMemo(
    () =>
      Object.entries(lookupVersionStatsMonthRaw ?? {}).map(([label, days]) => ({
        label,
        days,
        ...days.reduce(
          (acc, { day, total, uniqueIps }) => ({
            day,
            total: acc.total + total,
            uniqueIps: acc.uniqueIps + uniqueIps,
          }),
          { total: 0, uniqueIps: 0 },
        ),
      })),
    [lookupVersionStatsMonthRaw],
  );

  const lookupVersionStatsMonthMergedPercents = useMemo(
    () =>
      mergeLessThanPercent(
        Object.entries(lookupVersionStatsMonthRaw ?? {})
          .map(([label, days]) => ({
            label,
            total: days.reduce(
              (acc, data) => acc + data[lookupVersionStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'],
              0,
            ),
            uniqueIps: days.reduce((acc, { uniqueIps }) => acc + uniqueIps, 0),
          }))
          .sort((a, b) => b.total - a.total),
      ),
    [lookupVersionStatsMonthRaw, lookupVersionStatsMonthType],
  );

  return (
    <>
      <div className={'grid xl:grid-cols-2 grid-cols-1 gap-2 pb-2'}>
        <Card className={'p-4 flex-1 min-w-40 flex flex-row items-center justify-between'}>
          <ArchiveIcon className={'w-8 h-8'} />

          <div className={'flex flex-col text-right'}>
            <h1 className={'text-xl font-semibold'}>
              {stats?.size.total.jar !== undefined ? bytes(stats.size.total.jar) : <Skeleton className={'w-20 h-7'} />}
            </h1>
            <p className={'text-sm text-muted-foreground'}>Total Jar Size</p>
          </div>
        </Card>
        <Card className={'p-4 flex-1 min-w-40 flex flex-row items-center justify-between'}>
          <ArchiveRestoreIcon className={'w-8 h-8'} />

          <div className={'flex flex-col text-right'}>
            <h1 className={'text-xl font-semibold'}>
              {stats?.size.total.zip !== undefined ? bytes(stats.size.total.zip) : <Skeleton className={'w-20 h-7'} />}
            </h1>
            <p className={'text-sm text-muted-foreground'}>Total Zip Size</p>
          </div>
        </Card>

        <Card className={'p-4 h-[500px]'}>
          <div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
            <h1 className={'text-xl font-semibold ml-2'}>All Time Request Statistics for Versions</h1>

            <div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
              <Select
                value={requestStatsAllTimeType ?? 'total'}
                onValueChange={(value) => setRequestStatsAllTimeType(value)}
              >
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

          {!requestStatsAllTime.length ? (
            <div className={'w-full h-full flex flex-row items-center justify-center'}>
              <LoaderCircle className={'animate-spin'} />
            </div>
          ) : (
            <ChartContainer config={{}} className={'w-full h-full'}>
              <PieChart accessibilityLayer>
                <ChartTooltip content={<ChartTooltipContent />} />
                <Pie
                  data={requestStatsAllTime}
                  dataKey={'total'}
                  nameKey={'label'}
                  fillRule={'evenodd'}
                  label={({ name }) => name}
                >
                  {requestStatsAllTime.map(({ label }, i) => (
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
              <Select
                value={lookupVersionStatsAllTimeType ?? 'total'}
                onValueChange={(value) => setLookupVersionStatsAllTimeType(value)}
              >
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

          {!lookupVersionStatsAllTime.length ? (
            <div className={'w-full h-full flex flex-row items-center justify-center'}>
              <LoaderCircle className={'animate-spin'} />
            </div>
          ) : (
            <ChartContainer config={{}} className={'w-full h-full'}>
              <PieChart accessibilityLayer>
                <ChartTooltip content={<ChartTooltipContent />} />
                <Pie
                  data={lookupVersionStatsAllTime}
                  dataKey={'total'}
                  nameKey={'label'}
                  fillRule={'evenodd'}
                  label={({ name }) => name}
                >
                  {lookupVersionStatsAllTime.map(({ label }, i) => (
                    <Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
                  ))}
                </Pie>
              </PieChart>
            </ChartContainer>
          )}
        </Card>

        <Card className={'p-4 h-[500px]'}>
          <div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
            <h1 className={'text-xl font-semibold ml-2'}>
              Monthly Request Statistics for{' '}
              {!requestStatsMonthDisplay || requestStatsMonthDisplay === 'all'
                ? 'Versions'
                : requestStatsMonthDisplay === 'root'
                  ? 'Root'
                  : requestStatsMonthDisplay}
            </h1>

            <div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
              <SearchableSelect
                value={requestStatsMonthDisplay ?? 'all'}
                onChange={(value) => setRequestStatsMonthDisplay(value)}
                triggerClassName={'w-[8em] mb-1'}
                options={[
                  { value: 'all', label: 'All' },
                  { value: 'root', label: 'Root' },
                  ...(requestStatsMonth
                    ?.flatMap(({ versions }) => Object.entries(versions))
                    .filter(([value], index, self) => self.findIndex(([v]) => v === value) === index)
                    .sort(([_, a], [__, b]) => b - a)
                    .map(([label]) => ({ value: label, label })) ?? []),
                ]}
              />

              <Select
                value={requestStatsMonthType ?? 'total'}
                onValueChange={(value) => setRequestStatsMonthType(value)}
              >
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

          <div className={'absolute left-0 bottom-0 flex flex-row items-center justify-between p-2 z-10'}>
            <Select
              value={String(requestStatsMonthYear ?? new Date().getFullYear())}
              onValueChange={(value) => setRequestStatsMonthYear(Number(value))}
            >
              <SelectTrigger className={'w-[6em] mr-1'}>
                <SelectValue placeholder={'Year'} />
              </SelectTrigger>
              <SelectContent>
                {Array.from({ length: new Date().getFullYear() - 2023 }, (_, i) => 2024 + i).map((year) => (
                  <SelectItem key={year} value={String(year)}>
                    {year}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={String(requestStatsMonthMonth ?? new Date().getMonth() + 1)}
              onValueChange={(value) => setRequestStatsMonthMonth(Number(value))}
            >
              <SelectTrigger className={'w-[10em]'}>
                <SelectValue placeholder={'Month'} />
              </SelectTrigger>
              <SelectContent>
                {Array.from({ length: 12 }, (_, i) => i + 1).map((month) => (
                  <SelectItem key={month} value={String(month)}>
                    {Intl.DateTimeFormat('en-US', { month: 'long' }).format(new Date(2023, month - 1))}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {!requestStatsMonth?.length ||
          requestStatsMonth.reduce(
            (acc, { root, versions }) => acc + root + Object.values(versions).reduce((acc, total) => acc + total, 0),
            0,
          ) === 0 ? (
            requestStatsMonthRaw ? (
              <div className={'w-full h-full flex flex-row items-center justify-center'}>
                <p className={'text-muted-foreground'}>No data available for this month.</p>
              </div>
            ) : (
              <div className={'w-full h-full flex flex-row items-center justify-center'}>
                <LoaderCircle className={'animate-spin'} />
              </div>
            )
          ) : (
            <ChartContainer
              config={{}}
              className={'w-full md:h-[calc(100%-5rem)] h-[calc(100%-13rem)] absolute bottom-8 -left-4'}
            >
              {!requestStatsMonthDisplay || requestStatsMonthDisplay === 'all' ? (
                <PieChart accessibilityLayer>
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Pie
                    data={requestStatsMonthMergedPercents}
                    dataKey={'total'}
                    nameKey={'label'}
                    fillRule={'evenodd'}
                    label={({ name }) => name}
                  >
                    {requestStatsMonth.map(({ day }, i) => (
                      <Cell key={day} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
                    ))}
                  </Pie>
                </PieChart>
              ) : requestStatsMonthDisplay === 'root' ? (
                <BarChart data={requestStatsMonth} layout={'horizontal'}>
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <CartesianGrid vertical={false} />
                  <YAxis dataKey={'root'} type={'number'} />
                  <XAxis dataKey={'day'} type={'category'} />
                  <Bar fill={'hsl(var(--chart-1))'} dataKey={'root'} barSize={32} radius={2} />
                </BarChart>
              ) : (
                <BarChart
                  data={requestStatsMonth.map(({ day, versions }) => ({
                    day,
                    total: versions[requestStatsMonthDisplay ?? ''] ?? 0,
                  }))}
                  layout={'horizontal'}
                >
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <CartesianGrid vertical={false} />
                  <YAxis dataKey={'total'} type={'number'} />
                  <XAxis dataKey={'day'} type={'category'} />
                  <Bar fill={'hsl(var(--chart-1))'} dataKey={'total'} barSize={32} radius={2} />
                </BarChart>
              )}
            </ChartContainer>
          )}
        </Card>

        <Card className={'p-4 h-[500px]'}>
          <div className={'absolute left-0 w-full top-0 flex flex-row items-center justify-between p-2 z-10'}>
            <h1 className={'text-xl font-semibold ml-2'}>
              Monthly Lookup Statistics for{' '}
              {!lookupVersionStatsMonthDisplay || lookupVersionStatsMonthDisplay === 'all'
                ? 'Versions'
                : lookupVersionStatsMonthDisplay}
            </h1>

            <div className={'flex flex-row flex-wrap items-start justify-end self-start'}>
              <SearchableSelect
                value={lookupVersionStatsMonthDisplay ?? 'all'}
                onChange={(value) => setLookupVersionStatsMonthDisplay(value)}
                triggerClassName={'w-[8em] mb-1'}
                options={[
                  { value: 'all', label: 'All' },
                  ...lookupVersionStatsMonth.map(({ label }) => ({ value: label, label })),
                ]}
              />

              <Select
                value={lookupVersionStatsMonthType ?? 'total'}
                onValueChange={(value) => setLookupVersionStatsMonthType(value)}
              >
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

          <div className={'absolute left-0 bottom-0 flex flex-row items-center justify-between p-2 z-10'}>
            <Select
              value={String(lookupVersionStatsMonthYear ?? new Date().getFullYear())}
              onValueChange={(value) => setLookupVersionStatsMonthYear(Number(value))}
            >
              <SelectTrigger className={'w-[6em] mr-1'}>
                <SelectValue placeholder={'Year'} />
              </SelectTrigger>
              <SelectContent>
                {Array.from({ length: new Date().getFullYear() - 2023 }, (_, i) => 2024 + i).map((year) => (
                  <SelectItem key={year} value={String(year)}>
                    {year}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={String(lookupVersionStatsMonthMonth ?? new Date().getMonth() + 1)}
              onValueChange={(value) => setLookupVersionStatsMonthMonth(Number(value))}
            >
              <SelectTrigger className={'w-[10em]'}>
                <SelectValue placeholder={'Month'} />
              </SelectTrigger>
              <SelectContent>
                {Array.from({ length: 12 }, (_, i) => i + 1).map((month) => (
                  <SelectItem key={month} value={String(month)}>
                    {Intl.DateTimeFormat('en-US', { month: 'long' }).format(new Date(2023, month - 1))}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {!lookupVersionStatsMonth.length ? (
            lookupVersionStatsMonthRaw ? (
              <div className={'w-full h-full flex flex-row items-center justify-center'}>
                <p className={'text-muted-foreground'}>No data available for this month.</p>
              </div>
            ) : (
              <div className={'w-full h-full flex flex-row items-center justify-center'}>
                <LoaderCircle className={'animate-spin'} />
              </div>
            )
          ) : (
            <ChartContainer
              config={{}}
              className={'w-full md:h-[calc(100%-5rem)] h-[calc(100%-13rem)] absolute bottom-8 -left-4'}
            >
              {!lookupVersionStatsMonthDisplay || lookupVersionStatsMonthDisplay === 'all' ? (
                <PieChart accessibilityLayer>
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Pie
                    data={lookupVersionStatsMonthMergedPercents}
                    dataKey={'total'}
                    nameKey={'label'}
                    fillRule={'evenodd'}
                    label={({ name }) => name}
                  >
                    {lookupVersionStatsMonth.map(({ label }, i) => (
                      <Cell key={label} fill={`hsl(var(--chart-${(i % 5) + 1}))`} stroke={'hsl(var(--border))'} />
                    ))}
                  </Pie>
                </PieChart>
              ) : (
                <BarChart
                  data={lookupVersionStatsMonth
                    .filter(({ label }) => label === lookupVersionStatsMonthDisplay)
                    .map(({ days }) => days)
                    .flat()}
                  layout={'horizontal'}
                >
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <CartesianGrid vertical={false} />
                  <YAxis
                    dataKey={lookupVersionStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'}
                    type={'number'}
                  />
                  <XAxis dataKey={'day'} type={'category'} />
                  <Bar
                    fill={'hsl(var(--chart-1))'}
                    dataKey={lookupVersionStatsMonthType === 'uniqueIps' ? 'uniqueIps' : 'total'}
                    barSize={32}
                    radius={2}
                  />
                </BarChart>
              )}
            </ChartContainer>
          )}
        </Card>
      </div>
    </>
  );
}
