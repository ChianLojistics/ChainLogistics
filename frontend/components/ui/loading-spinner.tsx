import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface LoadingSpinnerProps extends React.SVGProps<SVGSVGElement> {
    size?: number;
}

export function LoadingSpinner({ size = 24, className, ...props }: LoadingSpinnerProps) {
    return (
        <Loader2
            size={size}
            className={cn("animate-spin text-muted-foreground", className)}
            {...props}
        />
    );
}
