package cli

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"runtime"

	"github.com/meowshed/meowctl/internal/version"
	"github.com/spf13/cobra"
)

const githubReleasesAPI = "https://api.github.com/repos/meowshed/meowctl/releases/latest"

type githubRelease struct {
	TagName string `json:"tag_name"`
	HTMLURL string `json:"html_url"`
	Assets  []struct {
		Name               string `json:"name"`
		BrowserDownloadURL string `json:"browser_download_url"`
	} `json:"assets"`
}

func newSelfUpdateCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "self-update",
		Short: "Update meowctl to the latest release",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return runSelfUpdate()
		},
	}
}

func fetchRelease(ctx context.Context) (*githubRelease, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, githubReleasesAPI, nil) // #nosec G107 -- fixed known URL
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: build request: %v", err)
	}
	resp, err := http.DefaultClient.Do(req) // #nosec G704 -- request URL is the fixed githubReleasesAPI constant
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: fetch releases: %v", err)
	}
	defer func() {
		if cerr := resp.Body.Close(); cerr != nil {
			fmt.Fprintf(os.Stderr, "self-update: close response body: %v\n", cerr)
		}
	}()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: read response: %v", err)
	}
	var release githubRelease
	if err := json.Unmarshal(body, &release); err != nil {
		return nil, exitErrorf(ExitModule, "self-update: parse release: %v", err)
	}
	return &release, nil
}

func downloadBinary(ctx context.Context, url string) ([]byte, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil) // #nosec G107 -- URL comes from trusted GitHub API response
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: build download request: %v", err)
	}
	resp, err := http.DefaultClient.Do(req) // #nosec G704 -- URL is from trusted GitHub releases API response; SRI verification deferred (TODO below)
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: download: %v", err)
	}
	defer func() {
		if cerr := resp.Body.Close(); cerr != nil {
			fmt.Fprintf(os.Stderr, "self-update: close download body: %v\n", cerr)
		}
	}()
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, exitErrorf(ExitModule, "self-update: read binary: %v", err)
	}
	return data, nil
}

func replaceBinary(data []byte) error {
	self, err := os.Executable()
	if err != nil {
		return exitErrorf(ExitGeneral, "self-update: locate binary: %v", err)
	}
	tmp, err := os.CreateTemp("", "meowctl-update-*") // #nosec G303
	if err != nil {
		return exitErrorf(ExitGeneral, "self-update: create temp file: %v", err)
	}
	tmpName := tmp.Name()
	defer func() { _ = os.Remove(tmpName) }()

	if _, err := tmp.Write(data); err != nil {
		_ = tmp.Close()
		return exitErrorf(ExitModule, "self-update: write binary: %v", err)
	}
	if err := tmp.Close(); err != nil {
		return exitErrorf(ExitModule, "self-update: close temp file: %v", err)
	}
	if err := os.Chmod(tmpName, 0o755); err != nil { // #nosec G703 G302 -- executable binary requires 0755; tmpName from os.CreateTemp
		return exitErrorf(ExitGeneral, "self-update: chmod: %v", err)
	}
	if err := os.Rename(tmpName, self); err != nil { // #nosec G703 -- self is os.Executable() path
		return exitErrorf(ExitGeneral, "self-update: replace binary: %v", err)
	}
	return nil
}

func runSelfUpdate() error {
	ctx := context.Background()
	fmt.Println("Checking for updates...")

	release, err := fetchRelease(ctx)
	if err != nil {
		return err
	}

	current := version.Version
	latest := release.TagName
	latestVer := latest
	if len(latestVer) > 0 && latestVer[0] == 'v' {
		latestVer = latestVer[1:]
	}

	if current == latestVer || (current == "dev" && latestVer == "") {
		fmt.Printf("Already up to date (%s).\n", current)
		return nil
	}

	wantName := fmt.Sprintf("meowctl_%s_%s_%s", latestVer, runtime.GOOS, runtime.GOARCH)
	var downloadURL string
	for _, a := range release.Assets {
		if a.Name == wantName {
			downloadURL = a.BrowserDownloadURL
			break
		}
	}
	if downloadURL == "" {
		return exitErrorf(ExitModule, "self-update: no binary asset found for %s/%s in release %s\n  See %s",
			runtime.GOOS, runtime.GOARCH, latest, release.HTMLURL)
	}

	fmt.Printf("Downloading %s (%s/%s)...\n", latest, runtime.GOOS, runtime.GOARCH)
	data, err := downloadBinary(ctx, downloadURL)
	if err != nil {
		return err
	}

	// TODO: verify SHA-256 of data against a .sha256 sidecar asset before replacing binary.
	if err := replaceBinary(data); err != nil {
		return err
	}

	fmt.Printf("Updated to %s. Run 'meowctl version' to confirm.\n", latest)
	return nil
}
