import React from 'react';

// Using async component in app router
export default async function VerifyPage(props: { params: Promise<{ id: string }> }) {
    const resolvedParams = await props.params;
    const productId = resolvedParams.id;

    return (
        <div className="min-h-screen bg-gradient-to-br from-gray-50 flex items-center justify-center p-4">
            <div className="max-w-xl w-full bg-white rounded-3xl shadow-xl overflow-hidden p-8 border border-gray-100">
                <div className="flex flex-col items-center">
                    <div className="w-20 h-20 bg-emerald-100 text-emerald-600 rounded-full flex items-center justify-center mb-6 ring-8 ring-emerald-50">
                        <svg className="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
                        </svg>
                    </div>
                    <h1 className="text-3xl font-bold text-gray-900 mb-2">Verified Authentic</h1>
                    <p className="text-gray-500 mb-8 text-center text-lg">This product&apos;s origin and journey are permanently recorded on the Stellar blockchain.</p>

                    <div className="w-full space-y-5 bg-gray-50 p-6 rounded-2xl">
                        <div className="flex justify-between items-center pb-4 border-b border-gray-200">
                            <span className="text-gray-500 font-medium tracking-wide text-sm uppercase">Product ID</span>
                            <span className="text-gray-900 font-mono text-sm bg-white px-3 py-1.5 rounded border border-gray-200 shadow-sm break-all text-right max-w-[60%]">
                                {productId}
                            </span>
                        </div>
                        <div className="flex justify-between items-center pb-2">
                            <span className="text-gray-500 font-medium tracking-wide text-sm uppercase">Current Status</span>
                            <span className="px-3 py-1 bg-emerald-100 border border-emerald-200 text-emerald-800 text-sm rounded-full font-semibold">
                                Ready for Sale
                            </span>
                        </div>
                    </div>

                    <div className="mt-8 pt-8 border-t border-gray-100 w-full">
                        <h3 className="font-bold text-xl mb-6 text-gray-800 flex items-center gap-2">
                            <svg className="w-5 h-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
                            </svg>
                            Supply Chain Journey
                        </h3>

                        {/* Timeline UI Placeholder */}
                        <div className="space-y-6 relative before:absolute before:inset-0 before:ml-5 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-gray-200 before:to-transparent">
                            <div className="relative flex items-center justify-between md:justify-normal md:odd:flex-row-reverse group is-active">
                                <div className="flex items-center justify-center w-10 h-10 rounded-full border border-white bg-indigo-100 text-indigo-600 shadow shrink-0 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2">
                                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                                    </svg>
                                </div>
                                <div className="w-[calc(100%-4rem)] md:w-[calc(50%-2.5rem)] p-4 rounded-xl border border-gray-100 bg-white shadow-sm">
                                    <div className="flex items-center justify-between mb-1">
                                        <h4 className="font-bold text-gray-900 text-lg">Product Registered</h4>
                                    </div>
                                    <p className="text-sm text-gray-500 font-medium">Origin info added to ledger</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
