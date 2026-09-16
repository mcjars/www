import { ChevronLeftIcon, ChevronRightIcon, PlusIcon, XIcon } from 'lucide-react';
import { Badge } from '~/components/ui/badge.tsx';
import { Button } from '~/components/ui/button.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu.tsx';

type OrganizationTypesInputProps = {
  value: string[];
  onChange: (types: string[]) => void;
  types: string[];
  disabled?: boolean;
};

export default function OrganizationTypesInput({ value, onChange, types, disabled }: OrganizationTypesInputProps) {
  const swap = (index: number, target: number) =>
    onChange(value.map((t, i) => (i === target ? value[index] : i === index ? value[target] : t)));

  return (
    <div className={'mt-2 flex flex-row flex-wrap items-center gap-2 rounded-md border border-input p-2 min-h-10'}>
      {value.map((type, index) => (
        <Badge key={type} variant={'secondary'} className={'flex flex-row items-center gap-1 pl-1 pr-1'}>
          <button
            type={'button'}
            disabled={disabled || index === 0}
            className={'disabled:opacity-30 hover:text-white'}
            onClick={() => swap(index, index - 1)}
          >
            <ChevronLeftIcon className={'w-4 h-4'} />
          </button>

          <span className={'font-mono'}>{type}</span>

          <button
            type={'button'}
            disabled={disabled || index === value.length - 1}
            className={'disabled:opacity-30 hover:text-white'}
            onClick={() => swap(index, index + 1)}
          >
            <ChevronRightIcon className={'w-4 h-4'} />
          </button>

          <button
            type={'button'}
            disabled={disabled}
            className={'ml-0.5 hover:text-red-400'}
            onClick={() => onChange(value.filter((_, i) => i !== index))}
          >
            <XIcon className={'w-4 h-4'} />
          </button>
        </Badge>
      ))}

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type={'button'}
            variant={'outline'}
            size={'sm'}
            className={'h-7'}
            disabled={disabled || types.every((t) => value.includes(t))}
          >
            <PlusIcon className={'w-4 h-4 mr-1'} />
            Add Type
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align={'start'} className={'max-h-64 overflow-y-auto'}>
          {types
            .filter((t) => !value.includes(t))
            .map((type) => (
              <DropdownMenuItem key={type} className={'font-mono'} onSelect={() => onChange([...value, type])}>
                {type}
              </DropdownMenuItem>
            ))}
        </DropdownMenuContent>
      </DropdownMenu>

      {!value.length && <span className={'text-sm text-gray-500'}>No types selected.</span>}
    </div>
  );
}
