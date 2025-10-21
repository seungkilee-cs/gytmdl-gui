export interface CookieValidationResult {
  is_valid: boolean;
  expiration_date?: string;
  days_until_expiry?: number;
  has_po_token: boolean;
  error?: string;
}

export interface CookieImportRequest {
  file_path: string;
}

export interface CookieImportResult {
  success: boolean;
  cookies_count?: number;
  error?: string;
}

export enum BrowserType {
  Chrome = "chrome",
  Firefox = "firefox",
  Safari = "safari",
  Edge = "edge",
  Opera = "opera",
}

export interface BrowserCookieInstructions {
  browser: BrowserType;
  steps: string[];
  cookie_path?: string;
}