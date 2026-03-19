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
    try {
      const member = await api.createMember(input);
      setMembers(prev => [...prev, member]);
      return member;
    } catch (e) {
      console.error("Failed to create member:", e);
      throw e;
    }
  };

  const update = async (id: number, input: Parameters<typeof api.updateMember>[1]) => {
    try {
      const member = await api.updateMember(id, input);
      setMembers(prev => prev.map(m => m.id === id ? member : m));
      return member;
    } catch (e) {
      console.error("Failed to update member:", e);
      throw e;
    }
  };

  const remove = async (id: number) => {
    try {
      await api.deleteMember(id);
      setMembers(prev => prev.filter(m => m.id !== id));
    } catch (e) {
      console.error("Failed to delete member:", e);
      throw e;
    }
  };

  return { members, loading, refresh, create, update, remove };
}
