export interface NewUser { email: string; age: number }

export function createUser(input: NewUser): NewUser {
  return { ...input };
}
