import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons';
import { useMemo, useState } from 'react';
import { Button } from '~/components/ui/button.tsx';
import { Input } from '~/components/ui/input.tsx';
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover.tsx';
import { cn } from '~/lib/utils.ts';

type Option = { value: string; label: string };

type SearchableSelectProps = {
  options: Option[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  triggerClassName?: string;
};

export function SearchableSelect({
  options,
  value,
  onChange,
  placeholder = 'Select',
  triggerClassName,
}: SearchableSelectProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');

  const filtered = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) return options;
    return options.filter((option) => option.label.toLowerCase().includes(query));
  }, [options, search]);

  const selectedLabel = options.find((option) => option.value === value)?.label ?? placeholder;

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
          <span className={'truncate'}>{selectedLabel}</span>
          <CaretSortIcon className={'ml-1 h-4 w-4 shrink-0 opacity-50'} />
        </Button>
      </PopoverTrigger>
      <PopoverContent className={'w-[14em] p-0'} align={'end'}>
        <div className={'p-1'}>
          <Input
            autoFocus
            placeholder={'Search...'}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            className={'h-8'}
          />
        </div>
        <div className={'max-h-60 overflow-y-auto p-1'}>
          {filtered.length === 0 && <p className={'px-2 py-1.5 text-sm text-muted-foreground'}>No results.</p>}
          {filtered.map((option) => (
            <button
              key={option.value}
              type={'button'}
              onClick={() => {
                onChange(option.value);
                setOpen(false);
              }}
              className={cn(
                'flex w-full items-center justify-between rounded-sm px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground',
                value === option.value && 'bg-accent text-accent-foreground',
              )}
            >
              <span className={'truncate'}>{option.label}</span>
              {value === option.value && <CheckIcon className={'ml-2 h-4 w-4 shrink-0'} />}
            </button>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
