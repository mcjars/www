import { LoaderCircle, PlusIcon } from 'lucide-react';
import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import apiGetTypes from '~/api/types.ts';
import apiCreateUserOrganization from '~/api/user/organization/create.ts';
import apiGetUserOrganizations, { Organization } from '~/api/user/organizations.ts';
import OrganizationRow from '~/components/organization-row.tsx';
import { Button } from '~/components/ui/button.tsx';
import { Input } from '~/components/ui/input.tsx';
import { useToast } from '~/hooks/use-toast.ts';
import { createMutator } from '~/lib/mutator.ts';

export default function PageOrganizations() {
  const [currentOrganization, setCurrentOrganization] = useState<Organization | null>(null);
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);

  const queryClient = useQueryClient();
  const { data: organizations } = useQuery({ queryKey: ['organizations'], queryFn: () => apiGetUserOrganizations() });
  const mutate = createMutator<NonNullable<typeof organizations>>(queryClient, ['organizations']);

  const { data: rawTypes } = useQuery({ queryKey: ['types'], queryFn: () => apiGetTypes() });

  const updateOrg = (organization: number) => {
    return (data: Partial<Organization>) => {
      mutate((organizations) => {
        if (!organizations) return;

        const org =
          organizations.owned.find((o) => o.id === organization) ??
          organizations.member.find((o) => o.id === organization);
        if (org) Object.assign(org, data);

        return { ...organizations };
      }, false);
    };
  };

  const types = Object.values(rawTypes ?? {})
    .flat()
    .map((t) => t.identifier);
  const { toast, toastError } = useToast();

  if (!organizations)
    return (
      <div className={'w-full mt-8 flex flex-row items-center justify-center'}>
        <LoaderCircle className={'animate-spin'} />
      </div>
    );

  return (
    <div className={'w-full pb-2 flex flex-col'}>
      <h1 className={'text-2xl font-semibold flex flex-row justify-between'}>
        <span className={'w-96'}>Owned Organizations</span>

        <form
          className={'flex flex-row'}
          onSubmit={(e) => {
            e.preventDefault();

            setLoading(true);

            const t = toast({
              title: 'Creating Organization...',
              description: `Creating ${name}.`,
            });

            apiCreateUserOrganization({ name })
              .then(() => {
                t.update(
                  toast({
                    title: 'Organization Created',
                    description: `${name} has been created.`,
                  }),
                );

                mutate();
                setName('');
              })
              .catch((error) => {
                t.update(
                  toastError({
                    title: 'Failed to Create Organization',
                    error,
                  }),
                );
              })
              .finally(() => setLoading(false));
          }}
        >
          <Input placeholder={'Name'} value={name} onChange={(e) => setName(e.target.value)} disabled={loading} />

          <Button type={'submit'} disabled={loading || !name} className={'ml-2'}>
            <PlusIcon className={'w-6 h-6 md:mr-2'} />
            <span className={'hidden md:inline'}>Create Organization</span>
          </Button>
        </form>
      </h1>
      {organizations?.owned.map((organization) => (
        <OrganizationRow
          key={organization.id}
          types={types}
          mutate={mutate}
          updateOrg={updateOrg(organization.id)}
          organization={organization}
          currentOrganization={currentOrganization}
          setCurrentOrganization={setCurrentOrganization}
        />
      ))}
      {!organizations?.owned.length && <p className={'text-gray-400 text-sm'}>You do not own any organizations.</p>}

      <h1 className={'text-2xl font-semibold mt-4'}>Member Organizations</h1>
      {organizations?.member.map((organization) => (
        <OrganizationRow
          key={organization.id}
          types={types}
          mutate={mutate}
          updateOrg={updateOrg(organization.id)}
          organization={organization}
          currentOrganization={currentOrganization}
          setCurrentOrganization={setCurrentOrganization}
        />
      ))}
      {!organizations?.member.length && (
        <p className={'text-gray-400 text-sm'}>You are not a member of any organizations.</p>
      )}

      <h1 className={'text-2xl font-semibold mt-4'}>Organization Invites</h1>
      {organizations?.invites.map((organization) => (
        <OrganizationRow
          key={organization.id}
          types={types}
          mutate={mutate}
          updateOrg={updateOrg(organization.id)}
          organization={organization}
          currentOrganization={currentOrganization}
          setCurrentOrganization={setCurrentOrganization}
          isPending
        />
      ))}
      {!organizations?.invites.length && (
        <p className={'text-gray-400 text-sm'}>You do not have any organization invites.</p>
      )}
    </div>
  );
}
