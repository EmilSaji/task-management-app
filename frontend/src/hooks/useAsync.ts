import { useCallback, useEffect, useRef, useState } from 'react';

export interface AsyncState<T> {
  data: T | null;
  error: unknown;
  loading: boolean;
  reload: () => Promise<void>;
}

/**
 * Runs `fn` on mount and exposes loading / error / data plus a `reload`.
 * Responses that arrive after unmount or after a newer call are ignored.
 */
export function useAsync<T>(fn: () => Promise<T>): AsyncState<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [loading, setLoading] = useState(true);
  const fnRef = useRef(fn);
  fnRef.current = fn;
  const callId = useRef(0);

  const reload = useCallback(async () => {
    const id = ++callId.current;
    setLoading(true);
    setError(null);
    try {
      const result = await fnRef.current();
      if (id === callId.current) setData(result);
    } catch (err) {
      if (id === callId.current) setError(err);
    } finally {
      if (id === callId.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
    return () => {
      callId.current++;
    };
  }, [reload]);

  return { data, error, loading, reload };
}
