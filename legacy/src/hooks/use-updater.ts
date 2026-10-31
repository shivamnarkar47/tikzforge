import { useState, useEffect } from "react";
import { checkForUpdates, UpdateInfo } from "../lib/updater";

export function useUpdater() {
  const [checking, setChecking] = useState(true);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    checkForUpdates()
      .then((info) => {
        setUpdateInfo(info);
      })
      .finally(() => {
        setChecking(false);
      });
  }, []);

  return { checking, updateInfo };
}
