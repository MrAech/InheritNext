import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";


export function cn(...imputs: ClassValue[]){
    return twMerge(clsx(imputs))
}