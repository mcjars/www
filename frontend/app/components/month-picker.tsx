import { ChevronLeftIcon, ChevronRightIcon } from 'lucide-react';
import { Button } from '~/components/ui/button.tsx';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select.tsx';

type MonthPickerProps = {
  year: number;
  month: number;
  onChange: (year: number, month: number) => void;
  minYear: number;
  descending?: boolean;
};

export default function MonthPicker({ year, month, onChange, minYear, descending }: MonthPickerProps) {
  const now = new Date();
  const currentYear = now.getFullYear();
  const currentMonth = now.getMonth() + 1;

  const years = Array.from({ length: currentYear - minYear + 1 }, (_, i) => minYear + i);
  if (descending) years.reverse();

  const step = (delta: number) => {
    const index = year * 12 + (month - 1) + delta;
    onChange(Math.floor(index / 12), (index % 12) + 1);
  };

  return (
    <>
      <Button
        variant={'outline'}
        size={'icon'}
        className={'mr-1 mb-1'}
        disabled={year < minYear || (year === minYear && month <= 1)}
        onClick={() => step(-1)}
      >
        <ChevronLeftIcon className={'w-4 h-4'} />
      </Button>

      <Select value={String(year)} onValueChange={(value) => onChange(Number(value), month)}>
        <SelectTrigger className={'w-[6em] mr-1 mb-1'}>
          <SelectValue placeholder={'Year'} />
        </SelectTrigger>
        <SelectContent>
          {years.map((y) => (
            <SelectItem key={y} value={String(y)}>
              {y}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select value={String(month)} onValueChange={(value) => onChange(year, Number(value))}>
        <SelectTrigger className={'w-[10em] mr-1 mb-1'}>
          <SelectValue placeholder={'Month'} />
        </SelectTrigger>
        <SelectContent>
          {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
            <SelectItem key={m} value={String(m)}>
              {Intl.DateTimeFormat('en-US', { month: 'long' }).format(new Date(2023, m - 1))}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Button
        variant={'outline'}
        size={'icon'}
        className={'mr-1 mb-1'}
        disabled={year > currentYear || (year === currentYear && month >= currentMonth)}
        onClick={() => step(1)}
      >
        <ChevronRightIcon className={'w-4 h-4'} />
      </Button>
    </>
  );
}
