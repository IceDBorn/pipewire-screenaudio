export default function matchNode(a, b) {
  return (
    a.mediaName === b.mediaName &&
    a.applicationName === b.applicationName &&
    a.serial === b.serial
  );
}
