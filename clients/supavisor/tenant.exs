# Adds the tenant of the file in argv to the metadata database of Supavisor, as the HTTP API does after its version check. run.sh uses it only when the API rejects the version of the server. See tenant.py.
[file] = System.argv()
{:ok, _} = Application.ensure_all_started(:ecto_sql)
{:ok, _} = Supavisor.Repo.start_link()
{:ok, _} = Supavisor.Vault.start_link()
params = file |> File.read!() |> JSON.decode!()

{:ok, _} =
  %Supavisor.Tenants.Tenant{}
  |> Supavisor.Tenants.Tenant.changeset(params)
  |> Supavisor.Repo.insert()
