import { createContext, useContext, ReactNode, Dispatch, SetStateAction } from "react";
import type { Interpretation } from "@/types";

interface ReaderContextValue {
  paperId: string;
  interpretation: Interpretation;
  activeConceptId: string | null;
  setActiveConceptId: Dispatch<SetStateAction<string | null>>;
}

const ReaderContext = createContext<ReaderContextValue | null>(null);

export function ReaderProvider({
  paperId,
  interpretation,
  activeConceptId,
  setActiveConceptId,
  children,
}: Omit<ReaderContextValue, "setActiveConceptId"> & {
  setActiveConceptId: Dispatch<SetStateAction<string | null>>;
  children: ReactNode;
}) {
  return (
    <ReaderContext.Provider
      value={{ paperId, interpretation, activeConceptId, setActiveConceptId }}
    >
      {children}
    </ReaderContext.Provider>
  );
}

export function useReaderContext() {
  const ctx = useContext(ReaderContext);
  if (!ctx) {
    throw new Error("useReaderContext must be used within ReaderProvider");
  }
  return ctx;
}
