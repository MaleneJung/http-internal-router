package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
)

type RouterConfig struct {
	Port int16 `json:"port"`
	TLS  bool  `json:"tls"`
}

type FirewallRules map[string]string

type Config struct {
	Router   RouterConfig  `json:"router"`
	Firewall FirewallRules `json:"firewall"`
}

func (cfg *Config) firewallRulingHandler(w http.ResponseWriter, r *http.Request) {

	if strings.Count(r.URL.Path, "/") >= 2 {

		pathSplit := strings.SplitN(r.URL.Path, "/", 3)

		for ruleFrom, ruleTo := range cfg.Firewall {
			if strings.EqualFold(pathSplit[1], ruleFrom) {

				internalURL := ruleTo + "/" + pathSplit[2]

				internalRequest, err := http.NewRequest(r.Method, internalURL, r.Body)
				if err != nil {
					break
				}

				internalResponse, err := http.DefaultClient.Do(internalRequest)
				if err != nil {
					break
				}
				defer internalResponse.Body.Close()
				for key, values := range internalResponse.Header {
					for _, value := range values {
						w.Header().Add(key, value)
					}
				}
				w.WriteHeader(internalResponse.StatusCode)

				io.Copy(w, internalResponse.Body)

				return

			}
		}

	}

	fmt.Fprintf(w, "Blocked by Firewall: \"%s\"\n", r.URL.Path)

}

func main() {

	file, err := os.Open("config.json")
	if err != nil {
		log.Fatal("Config does not exist! Please create a \"config.json\".")
		return
	}
	defer file.Close()

	bytes, err := io.ReadAll(file)
	if err != nil {
		log.Fatal(err)
		return
	}

	var config Config
	if err := json.Unmarshal(bytes, &config); err != nil {
		log.Fatal(err)
		return
	}

	http.HandleFunc("/", config.firewallRulingHandler)

	fmt.Println("Server is running on port", config.Router.Port)
	if config.Router.TLS {
		if err := http.ListenAndServeTLS(":"+fmt.Sprint(config.Router.Port), "tls/certificate.pem", "tls/key.pem", nil); err != nil {
			fmt.Println("Error starting server: ", err)
		}
	} else {
		if err := http.ListenAndServe(":"+fmt.Sprint(config.Router.Port), nil); err != nil {
			fmt.Println("Error starting server: ", err)
		}
	}

}
