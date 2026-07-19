declare const process: {
  on: (event: "unhandledRejection", listener: (reason: unknown) => void) => void;
  off: (event: "unhandledRejection", listener: (reason: unknown) => void) => void;
};

export async function collectUnhandledRejections(
  action: () => Promise<void>,
): Promise<unknown[]> {
  const reasons: unknown[] = [];
  const onProcessRejection = (reason: unknown) => reasons.push(reason);
  const onWindowRejection = (event: PromiseRejectionEvent) => {
    event.preventDefault();
    reasons.push(event.reason);
  };

  process.on("unhandledRejection", onProcessRejection);
  window.addEventListener("unhandledrejection", onWindowRejection);

  try {
    await action();
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    return reasons;
  } finally {
    process.off("unhandledRejection", onProcessRejection);
    window.removeEventListener("unhandledrejection", onWindowRejection);
  }
}
