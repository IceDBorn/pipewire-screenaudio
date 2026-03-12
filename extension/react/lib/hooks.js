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
	const [data, setData] = useState(null);

	useEffect(() => {
		readLocalStorage(name).then((val) => setData(val));
	}, [name]);

	const setStoredData = (val) => {
		setData(val);
		updateLocalStorage(name, val);
	};

	return [data, setStoredData];
}
