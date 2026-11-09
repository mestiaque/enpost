import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Join class names, letting later Tailwind classes override earlier ones. */
export const cn = (...v: ClassValue[]) => twMerge(clsx(v));
