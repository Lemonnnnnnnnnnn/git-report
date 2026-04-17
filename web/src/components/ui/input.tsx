import * as React from "react";
import { cn } from "../../lib/utils";

export const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      ref={ref}
      className={cn(
        "flex h-10 w-full rounded-lg border border-border bg-white px-3 py-2 text-sm outline-none ring-offset-white placeholder:text-slate-400 focus-visible:ring-2 focus-visible:ring-primary",
        className
      )}
      {...props}
    />
  )
);
Input.displayName = "Input";
