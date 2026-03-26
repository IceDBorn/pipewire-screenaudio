import { useEffect, useRef, useState } from "react";
import { readLocalStorage, updateLocalStorage } from "./local-storage";
import { unreachable } from "./utils";
import { LocalStorageTypes, UseLocalStorageResult } from "./types";

export function useDidUpdateEffect(
	fn: () => () => void,
	inputs?: React.DependencyList,
) {
	const didMountRef = useRef(false);

	useEffect(() => {
		if (didMountRef.current) {
			return fn();
		}
		didMountRef.current = true;
	}, inputs);
}

export function useLocalStorage<T extends keyof LocalStorageTypes>(
	name: T,
): UseLocalStorageResult<LocalStorageTypes[T]> {
	const [data, setData] = useState<LocalStorageTypes[T] | null | undefined>(
		undefined,
	);
	const [isLoading, setIsLoading] = useState(true);

	useEffect(() => {
		readLocalStorage(name).then((val) => {
			setData(val);
			setIsLoading(false);
		});
	}, [name]);

	const setStoredData = (val: LocalStorageTypes[T]) => {
		setData(val);
		updateLocalStorage(name, val);
	};

	return isLoading
		? { isLoading, data: undefined, setData: undefined }
		: data !== undefined
			? {
					isLoading,
					data,
					setData: setStoredData,
				}
			: unreachable();
}
