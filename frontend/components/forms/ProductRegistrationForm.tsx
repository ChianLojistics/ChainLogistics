"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { FormStepIndicator } from "./FormStepIndicator";
import { useWalletStore } from "@/lib/state/wallet.store";
import { registerProductOnChain, ProductData } from "@/lib/contract/product";
import { Loader2, ArrowRight, ArrowLeft, CheckCircle2 } from "lucide-react";

const productSchema = z.object({
    id: z.string().min(1, "Product ID is required").max(64).regex(/^[a-zA-Z0-9-_]+$/, "Only alphanumeric, dashes, and underscores allowed"),
    name: z.string().min(1, "Name is required").max(128),
    origin: z.string().min(1, "Origin is required").max(256),
    description: z.string().max(2048).optional(),
    category: z.string().min(1, "Category is required").max(64),
});

type ProductFormValues = z.infer<typeof productSchema>;

const STEPS = [
    { id: 1, name: "Basic Info" },
    { id: 2, name: "Origin Details" },
    { id: 3, name: "Review" },
];

export function ProductRegistrationForm() {
    const [step, setStep] = useState(1);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [txHash, setTxHash] = useState<string | null>(null);

    const { publicKey, status: walletStatus } = useWalletStore();

    const {
        register,
        handleSubmit,
        trigger,
        getValues,
        formState: { errors },
    } = useForm<ProductFormValues>({
        resolver: zodResolver(productSchema),
        defaultValues: {
            category: "Electronics",
        },
    });

    const nextStep = async () => {
        let fieldsToValidate: (keyof ProductFormValues)[] = [];
        if (step === 1) fieldsToValidate = ["id", "name"];
        if (step === 2) fieldsToValidate = ["origin", "category", "description"];

        const isValid = await trigger(fieldsToValidate);
        if (isValid) setStep((s) => s + 1);
    };

    const prevStep = () => setStep((s) => s - 1);

    const onSubmit = async (data: ProductFormValues) => {
        if (walletStatus !== "connected" || !publicKey) {
            alert("Please connect your wallet first");
            return;
        }

        setIsSubmitting(true);
        try {
            const hash = await registerProductOnChain(publicKey, data);
            setTxHash(hash);
            setStep(4); // Success step
        } catch (err) {
            console.error(err);
            alert("Failed to register product");
        } finally {
            setIsSubmitting(false);
        }
    };

    if (step === 4) {
        return (
            <div className="text-center py-10 bg-white rounded-xl border p-8 shadow-sm">
                <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-green-100 mb-6">
                    <CheckCircle2 className="h-10 w-10 text-green-600" />
                </div>
                <h2 className="text-2xl font-bold text-zinc-900 mb-2">Registration Successful!</h2>
                <p className="text-zinc-600 mb-6">Your product has been registered on the Stellar blockchain.</p>
                <div className="bg-zinc-50 rounded-lg p-4 mb-8 text-left border">
                    <p className="text-xs font-mono text-zinc-500 break-all">Transaction Hash: {txHash}</p>
                </div>
                <button
                    onClick={() => window.location.href = "/dashboard"}
                    className="bg-blue-600 text-white px-8 py-3 rounded-lg font-semibold hover:bg-blue-700 transition-all"
                >
                    View Dashboard
                </button>
            </div>
        );
    }

    return (
        <div className="max-w-2xl mx-auto">
            <FormStepIndicator steps={STEPS} currentStep={step} />

            <form onSubmit={handleSubmit(onSubmit)} className="bg-white rounded-xl border p-8 shadow-sm">
                {step === 1 && (
                    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
                        <h2 className="text-xl font-semibold mb-6">Basic Product Information</h2>
                        <div>
                            <label className="block text-sm font-medium text-zinc-700 mb-1">Product ID</label>
                            <input
                                {...register("id")}
                                className="w-full px-4 py-2 rounded-lg border focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all"
                                placeholder="e.g. SKU-12345"
                            />
                            {errors.id && <p className="text-xs text-red-500 mt-1">{errors.id.message}</p>}
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-zinc-700 mb-1">Product Name</label>
                            <input
                                {...register("name")}
                                className="w-full px-4 py-2 rounded-lg border focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all"
                                placeholder="e.g. Premium Coffee Beans"
                            />
                            {errors.name && <p className="text-xs text-red-500 mt-1">{errors.name.message}</p>}
                        </div>
                    </div>
                )}

                {step === 2 && (
                    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
                        <h2 className="text-xl font-semibold mb-6">Origin & Category</h2>
                        <div>
                            <label className="block text-sm font-medium text-zinc-700 mb-1">Origin Location</label>
                            <input
                                {...register("origin")}
                                className="w-full px-4 py-2 rounded-lg border focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all"
                                placeholder="e.g. Ethiopia, Yirgacheffe"
                            />
                            {errors.origin && <p className="text-xs text-red-500 mt-1">{errors.origin.message}</p>}
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-zinc-700 mb-1">Category</label>
                            <select
                                {...register("category")}
                                className="w-full px-4 py-2 rounded-lg border focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all bg-white"
                            >
                                <option value="Electronics">Electronics</option>
                                <option value="Food & Beverage">Food & Beverage</option>
                                <option value="Apparel">Apparel</option>
                                <option value="Industrial">Industrial</option>
                                <option value="Other">Other</option>
                            </select>
                            {errors.category && <p className="text-xs text-red-500 mt-1">{errors.category.message}</p>}
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-zinc-700 mb-1">Description (Optional)</label>
                            <textarea
                                {...register("description")}
                                rows={4}
                                className="w-full px-4 py-2 rounded-lg border focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all"
                                placeholder="Describe the product details..."
                            />
                            {errors.description && <p className="text-xs text-red-500 mt-1">{errors.description.message}</p>}
                        </div>
                    </div>
                )}

                {step === 3 && (
                    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
                        <h2 className="text-xl font-semibold mb-6">Review Registration Info</h2>
                        <div className="grid grid-cols-2 gap-4 text-sm bg-zinc-50 p-4 rounded-lg">
                            <span className="text-zinc-500 font-medium">Product ID:</span>
                            <span className="text-zinc-900 font-semibold">{getValues("id")}</span>

                            <span className="text-zinc-500 font-medium">Name:</span>
                            <span className="text-zinc-900 font-semibold">{getValues("name")}</span>

                            <span className="text-zinc-500 font-medium">Origin:</span>
                            <span className="text-zinc-900 font-semibold">{getValues("origin")}</span>

                            <span className="text-zinc-500 font-medium">Category:</span>
                            <span className="text-zinc-900 font-semibold">{getValues("category")}</span>
                        </div>
                        {getValues("description") && (
                            <div className="bg-zinc-50 p-4 rounded-lg">
                                <p className="text-sm text-zinc-500 font-medium mb-1">Description:</p>
                                <p className="text-sm text-zinc-700">{getValues("description")}</p>
                            </div>
                        )}

                        {walletStatus !== "connected" && (
                            <div className="bg-orange-50 border border-orange-200 rounded-lg p-4 text-sm text-orange-800">
                                Please connect your wallet to submit this registration.
                            </div>
                        )}
                    </div>
                )}

                <div className="mt-10 flex items-center justify-between border-t pt-6">
                    {step > 1 ? (
                        <button
                            type="button"
                            onClick={prevStep}
                            className="flex items-center gap-2 text-sm font-medium text-zinc-600 hover:text-zinc-900 transition-colors"
                        >
                            <ArrowLeft className="h-4 w-4" /> Back
                        </button>
                    ) : (
                        <div />
                    )}

                    {step < 3 ? (
                        <button
                            type="button"
                            onClick={nextStep}
                            className="bg-zinc-900 text-white px-6 py-2 rounded-lg font-semibold hover:bg-zinc-800 transition-all flex items-center gap-2"
                        >
                            Next <ArrowRight className="h-4 w-4" />
                        </button>
                    ) : (
                        <button
                            type="submit"
                            disabled={isSubmitting || walletStatus !== "connected"}
                            className="bg-blue-600 text-white px-8 py-2 rounded-lg font-semibold hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
                        >
                            {isSubmitting ? (
                                <>
                                    <Loader2 className="h-4 w-4 animate-spin" /> Submitting...
                                </>
                            ) : (
                                "Register Product"
                            )}
                        </button>
                    )}
                </div>
            </form>
        </div>
    );
}
