import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons';
import { useState } from 'react';
import { useDebounceValue } from 'usehooks-ts';
import apiGetVersions from '~/api/versions.ts';
import { Button } from '~/components/ui/button.tsx';
import { Input } from '~/components/ui/input.tsx';
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover.tsx';
import { useQuery } from '@tanstack/react-query';
import { cn } from '~/lib/utils.ts';

type VersionComboboxProps = {
  type: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  triggerClassName?: string;
};

export function VersionCombobox({
  type,
  value,
  onChange,
  placeholder = 'Version',
  triggerClassName,
}: VersionComboboxProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [debouncedSearch] = useDebounceValue(search, 300);

  const { data, isLoading } = useQuery({
    queryKey: ['version-combobox', type, debouncedSearch],
    queryFn: () => apiGetVersions(type, { page: 1, perPage: 50, search: debouncedSearch }),
    enabled: open,
  });

  const items = data?.items ?? [];

  return (
    <Popover
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) setSearch('');
      }}
    >
      <PopoverTrigger asChild>
        <Button
          variant={'outline'}
          role={'combobox'}
          aria-expanded={open}
          className={cn('justify-between font-normal', triggerClassName)}
        >
          <span className={'truncate'}>{value || placeholder}</span>
          <CaretSortIcon className={'ml-1 h-4 w-4 shrink-0 opacity-50'} />
        </Button>
      </PopoverTrigger>
      <PopoverContent className={'w-[14em] p-0'} align={'end'}>
        <div className={'p-1'}>
          <Input
            autoFocus
            placeholder={'Search versions...'}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            className={'h-8'}
          />
        </div>
        <div className={'max-h-60 overflow-y-auto p-1'}>
          {isLoading && <p className={'px-2 py-1.5 text-sm text-muted-foreground'}>Loading...</p>}
          {!isLoading && items.length === 0 && (
            <p className={'px-2 py-1.5 text-sm text-muted-foreground'}>No versions found.</p>
          )}
          {items.map(({ latest }) => {
            const versionId = latest.versionId;
            if (!versionId) return null;

            return (
              <button
                key={versionId}
                type={'button'}
                onClick={() => {
                  onChange(versionId);
                  setOpen(false);
                }}
                className={cn(
                  'flex w-full items-center justify-between rounded-sm px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground',
                  value === versionId && 'bg-accent text-accent-foreground',
                )}
              >
                <span className={'truncate'}>{versionId}</span>
                {value === versionId && <CheckIcon className={'ml-2 h-4 w-4 shrink-0'} />}
              </button>
            );
          })}
        </div>
      </PopoverContent>
    </Popover>
  );
}
