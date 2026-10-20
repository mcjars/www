import { BuildingIcon, PencilIcon, TrashIcon, WebhookIcon } from 'lucide-react';
import { useState } from 'react';
import apiDeleteUserAdminWebhook from '~/api/user/admin/webhook/delete.ts';
import apiPatchUserAdminWebhook from '~/api/user/admin/webhook/patch.ts';
import { Webhook } from '~/api/user/admin/webhooks.ts';
import { Badge } from '~/components/ui/badge.tsx';
import { Button } from '~/components/ui/button.tsx';
import { Card } from '~/components/ui/card.tsx';
import { Dialog, DialogContent, DialogTitle } from '~/components/ui/dialog.tsx';
import { Switch } from '~/components/ui/switch.tsx';
import WebhookDialog, { webhookData } from '~/components/webhook-dialog.tsx';
import { useToast } from '~/hooks/use-toast.ts';

const maskUrl = (url: string) => url.replace(/\/[A-Za-z0-9_-]{68}$/, '/••••••••');

type WebhookRowProps = {
  webhook: Webhook;
  types: string[];
  updateWebhook: (webhook: Webhook) => void;
  mutate: () => void;
};

export default function WebhookRow({ webhook, types, updateWebhook, mutate }: WebhookRowProps) {
  const [view, setView] = useState<'edit' | 'delete'>();
  const [loading, setLoading] = useState(false);

  const { toast, toastError } = useToast();

  return (
    <>
      <WebhookDialog
        open={view === 'edit'}
        onOpenChange={(open) => !open && setView(undefined)}
        webhook={webhook}
        types={types}
        onSaved={(saved) => saved && updateWebhook(saved)}
      />

      <Dialog open={view === 'delete'} onOpenChange={(open) => !open && setView(undefined)}>
        <DialogContent className={'md:min-w-[40rem]'}>
          <DialogTitle>Delete Webhook</DialogTitle>

          <p className={'text-sm text-gray-500'}>
            Are you sure you want to delete <b>{webhook.label ?? `webhook #${webhook.id}`}</b>? This action is
            irreversible.
          </p>

          <Button
            variant={'destructive'}
            className={'mt-4'}
            disabled={loading}
            onClick={() => {
              setLoading(true);

              apiDeleteUserAdminWebhook(webhook.id)
                .then(() => {
                  toast({
                    title: 'Webhook Deleted',
                    description: 'The webhook has been deleted.',
                  });

                  setView(undefined);
                  mutate();
                })
                .catch((error) => {
                  toastError({
                    title: 'Failed to Delete Webhook',
                    error,
                  });
                })
                .finally(() => setLoading(false));
            }}
          >
            Delete
          </Button>
        </DialogContent>
      </Dialog>

      <Card className={'p-3 mb-2'}>
        <div className={'flex flex-row flex-wrap items-center justify-between gap-2'}>
          <div className={'flex flex-col min-w-0'}>
            <div className={'flex flex-row items-center gap-2 min-w-0'}>
              <WebhookIcon className={'w-4 h-4 text-muted-foreground shrink-0'} />
              <span className={'text-sm font-semibold truncate'}>{webhook.label ?? `#${webhook.id}`}</span>
              {webhook.organization && (
                <Badge variant={'secondary'} className={'flex flex-row items-center gap-1'}>
                  <BuildingIcon size={12} />
                  {webhook.organization.name}
                  {webhook.organization.verified && ' ✓'}
                </Badge>
              )}
              {!webhook.enabled && <Badge className={'bg-red-500 hover:bg-red-400'}>disabled</Badge>}
            </div>

            <span className={'font-mono text-xs text-muted-foreground truncate mt-1'} title={maskUrl(webhook.url)}>
              {maskUrl(webhook.url)}
            </span>
          </div>

          <div className={'flex flex-row items-center gap-4'}>
            <div className={'flex flex-col items-end'}>
              <span className={'text-sm font-semibold'}>
                {webhook.successful.toLocaleString()}
                <span className={'text-muted-foreground font-normal'}> / </span>
                <span className={webhook.failed > 0 ? 'text-red-400' : undefined}>
                  {webhook.failed.toLocaleString()}
                </span>
              </span>
              <span className={'text-xs text-muted-foreground'}>successful / failed</span>
            </div>

            <Switch
              disabled={loading}
              checked={webhook.enabled}
              title={webhook.enabled ? 'Disable' : 'Enable'}
              onCheckedChange={(enabled) => {
                setLoading(true);

                apiPatchUserAdminWebhook(webhook.id, { ...webhookData(webhook), enabled })
                  .then(updateWebhook)
                  .catch((error) => {
                    toastError({
                      title: 'Failed to Update Webhook',
                      error,
                    });
                  })
                  .finally(() => setLoading(false));
              }}
            />

            <Button variant={'outline'} size={'sm'} disabled={loading} onClick={() => setView('edit')}>
              <PencilIcon size={16} />
            </Button>
            <Button variant={'destructive'} size={'sm'} disabled={loading} onClick={() => setView('delete')}>
              <TrashIcon size={16} />
            </Button>
          </div>
        </div>

        <div className={'flex flex-row flex-wrap gap-1 mt-2'}>
          {webhook.types.map((type) => (
            <Badge key={type} variant={'secondary'} className={'text-xs font-mono'}>
              {type}
            </Badge>
          ))}
        </div>
      </Card>
    </>
  );
}
