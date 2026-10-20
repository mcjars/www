import { useEffect, useState } from 'react';
import apiCreateUserAdminWebhook, { WebhookData } from '~/api/user/admin/webhook/create.ts';
import apiPatchUserAdminWebhook from '~/api/user/admin/webhook/patch.ts';
import { Webhook } from '~/api/user/admin/webhooks.ts';
import OrganizationTypesInput from '~/components/organization-types-input.tsx';
import { Button } from '~/components/ui/button.tsx';
import { Dialog, DialogContent, DialogTitle } from '~/components/ui/dialog.tsx';
import { Input } from '~/components/ui/input.tsx';
import { Label } from '~/components/ui/label.tsx';
import { Switch } from '~/components/ui/switch.tsx';
import { useToast } from '~/hooks/use-toast.ts';

export const WEBHOOK_URL_PATTERN =
  /^https:\/\/(?:ptb\.|canary\.)?discord\.com\/api(?:\/v[0-9]{1,2})?\/webhooks\/[0-9]{17,19}\/[A-Za-z0-9_-]{68}$/;

export const webhookData = (webhook: Webhook): WebhookData => ({
  organization: webhook.organization?.id ?? null,
  label: webhook.label,
  url: webhook.url,
  types: webhook.types,
  enabled: webhook.enabled,
});

type WebhookDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  webhook?: Webhook;
  types: string[];
  onSaved: (webhook?: Webhook) => void;
};

export default function WebhookDialog({ open, onOpenChange, webhook, types, onSaved }: WebhookDialogProps) {
  const [loading, setLoading] = useState(false);
  const [organization, setOrganization] = useState('');
  const [data, setData] = useState<Omit<WebhookData, 'organization'>>({
    label: null,
    url: '',
    types,
    enabled: true,
  });
  const [resetCounters, setResetCounters] = useState(false);

  const { toast, toastError } = useToast();

  useEffect(() => {
    if (!open) return;

    setOrganization(webhook?.organization ? String(webhook.organization.id) : '');
    setData(webhook ? webhookData(webhook) : { label: null, url: '', types, enabled: true });
    setResetCounters(false);
  }, [open, webhook, types]);

  const urlValid = WEBHOOK_URL_PATTERN.test(data.url);
  const organizationValid = organization === '' || /^[1-9][0-9]*$/.test(organization);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={'md:min-w-[40rem]'}>
        <DialogTitle>{webhook ? 'Edit Webhook' : 'Create Webhook'}</DialogTitle>

        <form
          className={'w-full grid grid-cols-1 md:grid-cols-2 gap-2'}
          onSubmit={(e) => {
            e.preventDefault();

            setLoading(true);

            const payload: WebhookData = {
              ...data,
              label: data.label?.trim() || null,
              organization: organization === '' ? null : Number(organization),
            };

            const request = webhook
              ? apiPatchUserAdminWebhook(webhook.id, { ...payload, resetCounters })
              : apiCreateUserAdminWebhook(payload).then(() => undefined);

            request
              .then((saved) => {
                toast({
                  title: webhook ? 'Webhook Updated' : 'Webhook Created',
                  description: webhook ? 'The webhook has been updated.' : 'The webhook has been created.',
                });

                onSaved(saved);
                onOpenChange(false);
              })
              .catch((error) => {
                toastError({
                  title: webhook ? 'Failed to Update Webhook' : 'Failed to Create Webhook',
                  error,
                });
              })
              .finally(() => setLoading(false));
          }}
        >
          <span className={'flex flex-col'}>
            <Label htmlFor={'webhook.label'}>Label</Label>
            <Input
              id={'webhook.label'}
              disabled={loading}
              className={'mt-2'}
              placeholder={'Label (optional)'}
              value={data.label ?? ''}
              maxLength={255}
              autoComplete={'off'}
              onChange={(e) => setData({ ...data, label: e.target.value })}
            />
          </span>
          <span className={'flex flex-col'}>
            <Label htmlFor={'webhook.organization'}>Organization ID</Label>
            <Input
              id={'webhook.organization'}
              disabled={loading}
              className={'mt-2'}
              placeholder={'Organization ID (optional)'}
              inputMode={'numeric'}
              value={organization}
              autoComplete={'off'}
              onChange={(e) => setOrganization(e.target.value.trim())}
            />
          </span>
          <span className={'flex flex-col col-span-full'}>
            <Label htmlFor={'webhook.url'}>Discord Webhook URL</Label>
            <Input
              id={'webhook.url'}
              disabled={loading}
              className={'mt-2 font-mono'}
              placeholder={'https://discord.com/api/webhooks/...'}
              value={data.url}
              maxLength={255}
              autoComplete={'off'}
              onChange={(e) => setData({ ...data, url: e.target.value.trim() })}
            />
            {data.url && !urlValid && (
              <p className={'text-xs text-red-400 mt-1'}>Must be a discord.com webhook URL.</p>
            )}
          </span>
          <span className={'flex flex-col col-span-full'}>
            <Label>Types</Label>
            <OrganizationTypesInput
              value={data.types}
              onChange={(types) => setData({ ...data, types })}
              types={types}
              disabled={loading}
            />
          </span>
          <span className={'flex flex-col'}>
            <Label htmlFor={'webhook.enabled'}>Enabled</Label>
            <div className={'mt-2 w-full h-full flex flex-row items-center'}>
              <Switch
                id={'webhook.enabled'}
                disabled={loading}
                checked={data.enabled}
                onCheckedChange={(checked) => setData({ ...data, enabled: checked })}
              />
            </div>
          </span>
          {webhook && (
            <span className={'flex flex-col'}>
              <Label htmlFor={'webhook.reset'}>Reset Counters</Label>
              <div className={'mt-2 w-full h-full flex flex-row items-center'}>
                <Switch
                  id={'webhook.reset'}
                  disabled={loading}
                  checked={resetCounters}
                  onCheckedChange={setResetCounters}
                />
              </div>
            </span>
          )}

          <Button
            type={'submit'}
            className={'col-span-full mt-2'}
            disabled={loading || !urlValid || !organizationValid || !data.types.length}
          >
            {webhook ? 'Save' : 'Create'}
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
