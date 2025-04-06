"use client"

import * as React from "react"
import * as TooltipPrimitive from "@radix-ui/react-tooltip"

import { useIsMobile } from "@/hooks/use-mobile"
import { cn } from "@/lib/utils"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"

const TooltipProvider = TooltipPrimitive.Provider

const Tooltip = TooltipPrimitive.Root

const TooltipTrigger = TooltipPrimitive.Trigger

const TooltipContent = React.forwardRef<
	React.ElementRef<typeof TooltipPrimitive.Content>,
	React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Content>
>(({ className, sideOffset = 4, ...props }, ref) => (
	<TooltipPrimitive.Portal>
		<TooltipPrimitive.Content
			ref={ref}
			sideOffset={sideOffset}
			className={cn(
				"z-50 overflow-hidden rounded-md bg-background border border-border p-1.5 text-xs text-foreground animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
				className
			)}
			{...props}
		/>
	</TooltipPrimitive.Portal>
))
TooltipContent.displayName = "TooltipContent"

const tooltipStyles = "z-50 overflow-hidden rounded-md bg-background border border-border p-1.5 text-xs text-foreground animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"

const ResponsiveTooltip: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
	children
}) => {
	const isMobile = useIsMobile()
	const [open, setOpen] = React.useState(false)
	if (isMobile) {
		let triggerChildren: React.ReactNode = null,
			contentChildren: React.ReactNode = null
		React.Children.forEach(children, (child) => {
			if (!React.isValidElement(child)) return
			if (child.type === Tooltip || (typeof child.type === 'object' && child.type !== null && 'displayName' in (child.type as any) && (child.type as any).displayName === 'Tooltip')) {
				React.Children.forEach(child.props.children, (tooltipChild) => {
					if (!React.isValidElement(tooltipChild)) return
					const childType =
						typeof tooltipChild.type === "object" && tooltipChild.type !== null && "displayName" in tooltipChild.type
							? (tooltipChild.type as any).displayName
							: tooltipChild.type
					if (childType === 'TooltipTrigger') {
						triggerChildren = (tooltipChild as React.ReactElement).props.children
					} else if (childType === 'TooltipContent') {
						contentChildren = (tooltipChild as React.ReactElement).props.children
					}
				})
			}
		})
		return (
			<Popover open={open} onOpenChange={setOpen}>
				{triggerChildren && (
					<PopoverTrigger onClick={() => setOpen(!open)}>
						{triggerChildren}
					</PopoverTrigger>
				)}
				{contentChildren && (
					<PopoverContent
						align="center"
						sideOffset={4}
						className={cn(
							tooltipStyles,
							"w-auto"
						)}
					>
						{contentChildren}
					</PopoverContent>
				)}
			</Popover>
		)
	}
	return <>{children}</>
}
ResponsiveTooltip.displayName = "ResponsiveTooltip"

export {
	TooltipProvider,
	Tooltip,
	TooltipTrigger,
	TooltipContent,
	ResponsiveTooltip
}