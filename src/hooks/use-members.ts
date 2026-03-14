import { useState, useEffect, useCallback } from "react";
import type { Member } from "@/types";
import * as api from "@/tauri/commands";

export function useMembers() {
  const [members, setMembers] = useState<Member[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await api.listMembers();
      setMembers(data);
    } catch (e) {
      console.error("Failed to load members", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const create = async (input: Parameters<typeof api.createMember>[0]) => {
    const member = await api.createMember(input);
    await refresh();
    return member;
  };

  const update = async (id: number, input: Parameters<typeof api.updateMember>[1]) => {
    const member = await api.updateMember(id, input);
    await refresh();
    return member;
  };

  const remove = async (id: number) => {
    await api.deleteMember(id);
    await refresh();
  };

  return { members, loading, refresh, create, update, remove };
}
