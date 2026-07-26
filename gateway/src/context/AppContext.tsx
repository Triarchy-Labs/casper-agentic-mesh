"use client";

import React, { createContext, useContext, useState, useEffect, ReactNode, useCallback } from "react";

interface AppContextType {
    connected: boolean;
    pubKey: string;
    connecting: boolean;
    walletMissing: boolean;
    showDisconnect: boolean;
    setShowDisconnect: React.Dispatch<React.SetStateAction<boolean>>;
    connectWallet: () => Promise<void>;
    disconnectWallet: () => void;
    meshLoad: number;
    setMeshLoad: React.Dispatch<React.SetStateAction<number>>;
    gasPrice: number;
    setGasPrice: React.Dispatch<React.SetStateAction<number>>;
    streamedTokens: number;
    setStreamedTokens: React.Dispatch<React.SetStateAction<number>>;
}

const checkCasperConnected = async () => {
    if (typeof window !== "undefined" && window.casperWallet) {
        try {
            const isConnected = await window.casperWallet.isConnected();
            return { isConnected };
        } catch {
            return { isConnected: false };
        }
    }
    return { isConnected: false };
};

const requestAccess = async () => {
    if (typeof window !== "undefined" && window.casperWallet) {
        try {
            await window.casperWallet.requestConnection();
            const activeKey = await window.casperWallet.getActivePublicKey();
            return { address: activeKey };
        } catch (error) {
            return { error };
        }
    }
    return { error: "Casper Wallet not installed" };
};

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
    const [connected, setConnected] = useState(false);
    const [pubKey, setPubKey] = useState("");
    const [connecting, setConnecting] = useState(false);
    const [walletMissing, setWalletMissing] = useState(false);
    const [showDisconnect, setShowDisconnect] = useState(false);

    const [meshLoad, setMeshLoad] = useState(74);
    const [gasPrice, setGasPrice] = useState(1.02);
    const [streamedTokens, setStreamedTokens] = useState(1048576);

    useEffect(() => {
        let isMounted = true;
        const checkConn = async () => {
            try {
                const status = await checkCasperConnected();
                if (status.isConnected && isMounted) {
                    const access = await requestAccess();
                    if (access.address && isMounted) {
                        const key = access.address;
                        setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
                        setConnected(true);
                    }
                }
            } catch (e) {
                console.warn("[AppContext] Casper wallet connection check failed:", e);
            }
        };
        checkConn();
        return () => {
            isMounted = false;
        };
    }, []);

    const connectWallet = useCallback(async () => {
        if (connected) {
            setShowDisconnect((prev) => !prev);
            return;
        }

        if (walletMissing) {
            window.open("https://www.casperwallet.io/", "_blank");
            return;
        }

        setConnecting(true);
        try {
            if (typeof window === "undefined" || !window.casperWallet) {
                setWalletMissing(true);
                return;
            }

            const access = await requestAccess();
            if (access.error) {
                console.log("Casper Wallet connection rejected.", access.error);
            } else if (access.address) {
                const key = access.address;
                setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
                setConnected(true);
            }
        } catch (error) {
            console.error("Casper Wallet connect failed", error);
        } finally {
            setConnecting(false);
        }
    }, [connected, walletMissing]);

    const disconnectWallet = useCallback(() => {
        setConnected(false);
        setPubKey("");
        setShowDisconnect(false);
    }, []);

    return (
        <AppContext.Provider
            value={{
                connected,
                pubKey,
                connecting,
                walletMissing,
                showDisconnect,
                setShowDisconnect,
                connectWallet,
                disconnectWallet,
                meshLoad,
                setMeshLoad,
                gasPrice,
                setGasPrice,
                streamedTokens,
                setStreamedTokens,
            }}
        >
            {children}
        </AppContext.Provider>
    );
}

export function useApp() {
    const context = useContext(AppContext);
    if (!context) {
        throw new Error("useApp must be used within an AppProvider");
    }
    return context;
}
