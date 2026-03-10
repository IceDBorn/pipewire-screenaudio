import { useEffect, useRef, useState } from "react";
import { readLocalStorage, updateLocalStorage } from "./local-storage";

export function useDidUpdateEffect(fn, inputs) {
  const didMountRef = useRef(false);

  useEffect(() => {
    if (didMountRef.current) {
      return fn();
    }
    didMountRef.current = true;
  }, inputs);
}

export function useLocalStorage(name) {
  const storedData = readLocalStorage(name);
  const [data, setData] = useState(storedData);

  useEffect(() => {
    updateLocalStorage(name, data);
  }, [data, setData]);

  return [data, setData];
}
