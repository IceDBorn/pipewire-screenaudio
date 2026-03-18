import { useEffect, useRef, useState } from "react";
import { readLocalStorage, updateLocalStorage } from "./local-storage";

/** @import { LocalStorageTypes, UseLocalStorageResult } from "./types" */

/**
 * @param {() => (() => void)} fn
 * @param {React.DependencyList} inputs?
 */
export function useDidUpdateEffect(fn, inputs) {
	const didMountRef = useRef(false);

	useEffect(() => {
		if (didMountRef.current) {
			return fn();
		}
		didMountRef.current = true;
	}, inputs);
}

/**
 * @template {keyof LocalStorageTypes} T
 * @param {T} name
 * @returns {UseLocalStorageResult<LocalStorageTypes[T]>}
 */
export function useLocalStorage(name) {
	/**
	 * @type {ReturnType<typeof useState<LocalStorageTypes[T] | null>>}
	 */
	const [data, setData] = useState(undefined);
	const [isLoading, setIsLoading] = useState(true);

	useEffect(() => {
		readLocalStorage(name).then((val) => {
			setData(val);
			setIsLoading(false);
		});
	}, [name]);

	/**
	 * @param {LocalStorageTypes[T]} val
	 */
	const setStoredData = (val) => {
		setData(val);
		updateLocalStorage(name, val);
	};

	return { isLoading, data, setData: setStoredData };
}
