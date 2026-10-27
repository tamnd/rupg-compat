# The connect program of the connect suite, with Postgrex.
# Postgrex does not read PGSSLMODE and PGSSLROOTCERT, so the program gives the TLS options. The certificate of the oracle is a self-signed root, and Erlang rejects a self-signed peer even when the root is trusted. So the verify function accepts a self-signed peer only when it is the certificate of PGSSLROOTCERT and its names have the host.
# The program writes the row of the query on one line, with one space between the values.
host = System.fetch_env!("PGHOST")
[{:Certificate, der, _}] = :public_key.pem_decode(File.read!(System.fetch_env!("PGSSLROOTCERT")))

reference =
  case :inet.parse_address(String.to_charlist(host)) do
    {:ok, ip} -> [ip: ip]
    _ -> [dns_id: String.to_charlist(host)]
  end

verify = fn
  cert, {:bad_cert, :selfsigned_peer}, state ->
    if :public_key.pkix_encode(:OTPCertificate, cert, :otp) == der and :public_key.pkix_verify_hostname(cert, reference),
      do: {:valid, state},
      else: {:fail, :selfsigned_peer}

  _, {:bad_cert, reason}, _ ->
    {:fail, reason}

  _, {:extension, _}, state ->
    {:unknown, state}

  _, _, state ->
    {:valid, state}
end

ssl = [verify: :verify_peer, cacerts: [der], verify_fun: {verify, nil}]
{:ok, conn} = Postgrex.start_link(ssl: ssl, pool_size: 1)
%Postgrex.Result{rows: [[user, ssl, version]]} = Postgrex.query!(conn, "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()", [])
IO.puts("#{user} #{if ssl, do: "t", else: "f"} #{version}")
GenServer.stop(conn)
